use crate::calibration::CalibrationState;
use crate::commands::library::{find_song_by_id, find_song_by_id_mut, load_library, save_library};
use crate::models::song::SongId;
use crate::models::tab::{BassicalTab, CalibrationMeta, TimingPoint};
use crate::persistence::storage;
use std::sync::Arc;
use tauri::State;

const BPM_MIN: f64 = 20.0;
const BPM_MAX: f64 = 400.0;
const OFFSET_MIN: f64 = 0.0;

pub struct CalibrationTapState(pub Arc<CalibrationState>);

/// Devuelve la posición audible calibrada (ms) del reloj de audio, leyendo
/// solo variables atómicas — sin bloquear `Mutex<AudioEngine>` (ADR-004).
#[tauri::command]
pub fn record_calibration_tap(state: State<CalibrationTapState>) -> Result<f64, String> {
    Ok(state.0.audible_position_ms())
}

/// Devuelve los timing points de una canción. Si no hay archivo (sin
/// calibrar) devuelve un vector vacío, no un error.
#[tauri::command]
pub fn get_calibration(song_id: SongId) -> Result<Vec<TimingPoint>, String> {
    let tab: Option<BassicalTab> = storage::read_song(song_id.as_str())?;
    Ok(tab.map(|t| t.timing_points).unwrap_or_default())
}

/// Guarda (reemplaza) todos los timing points de una canción. Crea o
/// actualiza el archivo `songs/<id>.bassical.json`, conservando `tab`/`practice`
/// si ya existieran. Sincroniza `Song.has_calibration` y `Song.bpm` en
/// `library.json`.
#[tauri::command]
pub fn save_timing_points(song_id: SongId, timing_points: Vec<TimingPoint>) -> Result<(), String> {
    validate_timing_points(&timing_points)?;

    let library = load_library()?;
    let song =
        find_song_by_id(&library, &song_id).ok_or_else(|| "Canción no encontrada".to_string())?;

    // Carga el contenedor existente para conservar tab/practice, o crea uno
    // nuevo solo con calibración.
    let mut tab: BassicalTab =
        storage::read_song::<BassicalTab>(song_id.as_str())?.unwrap_or_else(|| {
            BassicalTab::for_calibration(
                CalibrationMeta {
                    id: song_id.clone(),
                    title: song.title.clone(),
                    artist: song.artist.clone(),
                    audio_path: song.audio_path.clone(),
                },
                Vec::new(),
            )
        });

    tab.title = song.title.clone();
    tab.artist = song.artist.clone();
    tab.audio_path = song.audio_path.clone();
    tab.timing_points = timing_points.clone();

    storage::write_song(song_id.as_str(), &tab)?;

    // Sincroniza el índice de la biblioteca.
    let mut library = library;
    {
        let song = find_song_by_id_mut(&mut library, &song_id)
            .ok_or_else(|| "Canción no encontrada".to_string())?;
        let now = chrono::Utc::now().to_rfc3339();
        song.has_calibration = !timing_points.is_empty();
        song.bpm = timing_points.first().map(|tp| tp.bpm);
        song.updated_at = now;
    }
    save_library(&library)?;
    Ok(())
}

/// Elimina toda la calibración de una canción: borra el archivo
/// `songs/<id>.bassical.json` (y con él la tab/practice si los hubiera —
/// la calibración y la tab comparten contenedor, ADR-003) y pone
/// `has_calibration=false` y `bpm=null` en la biblioteca.
#[tauri::command]
pub fn clear_calibration(song_id: SongId) -> Result<(), String> {
    // Verifica que la canción exista.
    let mut library = load_library()?;
    if find_song_by_id(&library, &song_id).is_none() {
        return Err("Canción no encontrada".to_string());
    }

    storage::delete_song_file(song_id.as_str())?;

    let song = find_song_by_id_mut(&mut library, &song_id)
        .ok_or_else(|| "Canción no encontrada".to_string())?;
    song.has_calibration = false;
    song.bpm = None;
    song.updated_at = chrono::Utc::now().to_rfc3339();
    save_library(&library)?;
    Ok(())
}

/// Valida un conjunto de timing points. Reglas:
/// - offset >= 0
/// - BPM en [BPM_MIN, BPM_MAX]
/// - time signature válida (si está presente)
/// - offsets estrictamente crecientes (cada TP rige una sección distinta)
fn validate_timing_points(points: &[TimingPoint]) -> Result<(), String> {
    for tp in points {
        validate_single_timing_point(tp)?;
    }
    // Offsets estrictamente crecientes.
    for w in points.windows(2) {
        if w[0].offset_ms >= w[1].offset_ms {
            return Err(format!(
                "Los offsets deben ser estrictamente crecientes: {} >= {}",
                w[0].offset_ms, w[1].offset_ms
            ));
        }
    }
    Ok(())
}

fn validate_single_timing_point(tp: &TimingPoint) -> Result<(), String> {
    if tp.offset_ms < OFFSET_MIN {
        return Err(format!(
            "Offset inválido (debe ser >= 0): {} ms",
            tp.offset_ms
        ));
    }
    if !(BPM_MIN..=BPM_MAX).contains(&tp.bpm) {
        return Err(format!(
            "BPM inválido (debe estar entre {} y {}): {}",
            BPM_MIN, BPM_MAX, tp.bpm
        ));
    }
    if let Some(ref ts) = tp.time_signature {
        if !ts.is_valid() {
            return Err(format!(
                "Time signature inválida: {}/{}",
                ts.numerator, ts.denominator
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::library::{add_song, get_library, init_app, update_song};
    use crate::models::song::{ArtistName, AudioPath, SongTitle};
    use crate::models::tab::TimeSignature;
    use crate::persistence::storage;
    use std::fs;

    fn assert_tp(tp: &TimingPoint, offset_ms: f64, bpm: f64, ts: Option<TimeSignature>) {
        assert_eq!(tp.offset_ms, offset_ms);
        assert_eq!(tp.bpm, bpm);
        assert_eq!(tp.time_signature, ts);
    }

    fn assert_json_omits(json: &str, keys: &[&str]) {
        for key in keys {
            assert!(!json.contains(key), "expected {} omitted: {}", key, json);
        }
    }

    fn assert_json_contains(json: &str, keys: &[&str]) {
        for key in keys {
            assert!(json.contains(key), "expected {} in: {}", key, json);
        }
    }

    fn make_temp_dir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bassical_cal_test_{:?}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).unwrap();
        storage::set_data_dir(dir.clone());
        dir
    }

    fn cleanup(dir: &std::path::PathBuf) {
        storage::clear_data_dir();
        fs::remove_dir_all(dir).ok();
    }

    fn make_song(dir: &std::path::PathBuf, name: &str) -> crate::models::song::Song {
        let audio_file = dir.join(format!("{name}.mp3"));
        fs::write(&audio_file, "fake audio").unwrap();
        add_song(
            SongTitle::new(name.to_string()),
            Some(ArtistName::new("Artist".to_string())),
            AudioPath::new(audio_file.to_str().unwrap().to_string()),
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn test_get_calibration_empty_when_no_file() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let tps = get_calibration(song.id).unwrap();
        assert!(tps.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_get_calibration_not_found_errors() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let result = get_calibration(SongId::new("no-such-id".to_string()));
        // No hay archivo → Ok([]), no error (la inexistencia del archivo no es
        // error; la inexistencia de la canción sí se valida al guardar).
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_save_then_get_timing_points() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let tps = vec![
            TimingPoint::new(0.0, 120.0),
            TimingPoint::with_time_signature(60000.0, 90.0, TimeSignature::new(3, 4)),
        ];
        save_timing_points(song.id.clone(), tps.clone()).unwrap();

        let back = get_calibration(song.id.clone()).unwrap();
        assert_eq!(back.len(), 2);
        assert_tp(&back[0], 0.0, 120.0, None);
        assert_tp(&back[1], 60000.0, 90.0, Some(TimeSignature::new(3, 4)));

        // El archivo existe en la ruta correcta.
        assert!(storage::song_file(song.id.as_str()).exists());
        cleanup(&dir);
    }

    #[test]
    fn test_save_updates_library_has_calibration_and_bpm() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        assert!(!song.has_calibration);
        assert!(song.bpm.is_none());

        save_timing_points(song.id.clone(), vec![TimingPoint::new(1240.0, 132.5)]).unwrap();

        let lib = get_library().unwrap();
        let updated = lib.songs.iter().find(|s| s.id == song.id).unwrap();
        assert!(updated.has_calibration);
        assert_eq!(updated.bpm, Some(132.5));
        cleanup(&dir);
    }

    #[test]
    fn test_save_empty_clears_calibration_flag_but_keeps_file_clean() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        save_timing_points(song.id.clone(), vec![TimingPoint::new(0.0, 120.0)]).unwrap();

        // Guardar lista vacía desactiva la calibración y deja timing_points=[].
        save_timing_points(song.id.clone(), vec![]).unwrap();
        let lib = get_library().unwrap();
        let updated = lib.songs.iter().find(|s| s.id == song.id).unwrap();
        assert!(!updated.has_calibration);
        assert!(updated.bpm.is_none());
        let tps = get_calibration(song.id.clone()).unwrap();
        assert!(tps.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_clear_calibration_deletes_file_and_flags() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        save_timing_points(song.id.clone(), vec![TimingPoint::new(0.0, 120.0)]).unwrap();
        assert!(storage::song_file(song.id.as_str()).exists());

        clear_calibration(song.id.clone()).unwrap();
        assert!(!storage::song_file(song.id.as_str()).exists());

        let lib = get_library().unwrap();
        let updated = lib.songs.iter().find(|s| s.id == song.id).unwrap();
        assert!(!updated.has_calibration);
        assert!(updated.bpm.is_none());
        cleanup(&dir);
    }

    #[test]
    fn test_clear_calibration_idempotent() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        // Sin archivo previo: no error.
        assert!(clear_calibration(song.id.clone()).is_ok());
        cleanup(&dir);
    }

    #[test]
    fn test_save_preserves_tab_and_practice_when_present() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        // Simula un archivo previo con tab/practice (Sprint 5).
        let existing = BassicalTab::for_calibration(
            CalibrationMeta {
                id: song.id.clone(),
                title: SongTitle::new("Old Title".to_string()),
                artist: None,
                audio_path: song.audio_path.clone(),
            },
            vec![TimingPoint::new(0.0, 100.0)],
        );
        let mut existing = existing;
        existing.practice = Some(crate::models::tab::PracticeData {
            preferred_speed: 0.75,
            last_position_ms: 1234.0,
        });
        storage::write_song(song.id.as_str(), &existing).unwrap();

        // Re-guarda solo calibración: practice debe conservarse.
        save_timing_points(song.id.clone(), vec![TimingPoint::new(500.0, 110.0)]).unwrap();

        let back: BassicalTab = storage::read_song(song.id.as_str()).unwrap().unwrap();
        assert_eq!(back.timing_points.len(), 1);
        assert_tp(&back.timing_points[0], 500.0, 110.0, None);
        assert!(back.practice.is_some());
        assert_eq!(back.practice.unwrap().preferred_speed, 0.75);
        cleanup(&dir);
    }

    #[test]
    fn test_save_syncs_title_artist_audio_path_from_library() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        save_timing_points(song.id.clone(), vec![TimingPoint::new(0.0, 120.0)]).unwrap();

        // Edita el título en la biblioteca.
        let mut u = crate::commands::library::SongUpdate::default();
        u.title = Some(SongTitle::new("New Title".to_string()));
        update_song(song.id.clone(), u).unwrap();

        // Re-guarda calibración: el archivo debe reflejar el nuevo título.
        save_timing_points(song.id.clone(), vec![TimingPoint::new(0.0, 121.0)]).unwrap();

        let back: BassicalTab = storage::read_song(song.id.as_str()).unwrap().unwrap();
        assert_eq!(back.title.as_str(), "New Title");
        cleanup(&dir);
    }

    // --- Validación ---

    #[test]
    fn test_validate_rejects_negative_offset() {
        let pts = vec![TimingPoint::new(-1.0, 120.0)];
        assert!(validate_timing_points(&pts).is_err());
    }

    #[test]
    fn test_validate_rejects_bpm_out_of_range() {
        for bpm in [10.0, 500.0] {
            assert!(validate_timing_points(&[TimingPoint::new(0.0, bpm)]).is_err());
        }
        for bpm in [20.0, 400.0] {
            assert!(validate_timing_points(&[TimingPoint::new(0.0, bpm)]).is_ok());
        }
    }

    #[test]
    fn test_validate_rejects_invalid_time_signature() {
        let pts = vec![TimingPoint::with_time_signature(
            0.0,
            120.0,
            TimeSignature::new(4, 3),
        )];
        assert!(validate_timing_points(&pts).is_err());
    }

    #[test]
    fn test_validate_rejects_non_increasing_offsets() {
        let pts = vec![
            TimingPoint::new(1000.0, 120.0),
            TimingPoint::new(1000.0, 130.0),
        ];
        assert!(validate_timing_points(&pts).is_err());
    }

    #[test]
    fn test_validate_accepts_valid_set() {
        let pts = vec![
            TimingPoint::new(0.0, 120.0),
            TimingPoint::with_time_signature(60000.0, 90.0, TimeSignature::new(6, 8)),
            TimingPoint::new(120000.0, 140.0),
        ];
        assert!(validate_timing_points(&pts).is_ok());
    }

    #[test]
    fn test_save_rejects_invalid_points() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        let result = save_timing_points(
            song.id.clone(),
            vec![TimingPoint::new(0.0, 1.0)], // bpm too low
        );
        assert!(result.is_err());
        // No debe haber escrito archivo ni tocado la biblioteca.
        assert!(!storage::song_file(song.id.as_str()).exists());
        let lib = get_library().unwrap();
        let s = lib.songs.iter().find(|s| s.id == song.id).unwrap();
        assert!(!s.has_calibration);
        cleanup(&dir);
    }

    #[test]
    fn test_save_not_found_song_errors() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let result = save_timing_points(
            SongId::new("no-id".to_string()),
            vec![TimingPoint::new(0.0, 120.0)],
        );
        assert!(result.is_err());
        cleanup(&dir);
    }

    #[test]
    fn test_bassical_json_file_omits_tab_and_practice_in_sprint4() {
        let dir = make_temp_dir();
        init_app().unwrap();
        let song = make_song(&dir, "Song");
        save_timing_points(song.id.clone(), vec![TimingPoint::new(0.0, 120.0)]).unwrap();
        let raw = fs::read_to_string(storage::song_file(song.id.as_str())).unwrap();
        assert_json_omits(&raw, &["\"tab\"", "\"practice\""]);
        assert_json_contains(&raw, &["schemaVersion", "timingPoints"]);
        cleanup(&dir);
    }
}
