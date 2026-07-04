use aurora_core::ParameterBundle;

use aurora_engine::generate_composition;

#[test]
fn integration_generates_sixteen_bars_with_harmony_and_melody() {
    let mut params = ParameterBundle::default();
    params.style.genre = "classical".into();
    params.form.section_lengths = vec![16];
    params.form.section_count = 1;
    params.mode.key = 0;
    params.mode.mode = "major".into();
    params.rhythm.time_signature_beats = 4;
    params.rhythm.time_signature_beat_type = 4;
    params.search.beam_width = 8;

    let comp = generate_composition(params).expect("16-bar generation should succeed");

    let measures: Vec<_> = comp
        .movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
        .flat_map(|p| &p.measures)
        .collect();

    assert_eq!(measures.len(), 16, "expected 16 measures");

    for measure in &measures {
        assert!(
            !measure.harmony_slots.is_empty(),
            "measure {} missing harmony",
            measure.number.global
        );
        assert!(
            !measure.harmony_slots[0].provenance.rule_ids.is_empty()
                || measure.harmony_slots[0].provenance.stage.is_some(),
            "harmony slot missing provenance"
        );
    }

    let melody_notes: Vec<_> = measures
        .iter()
        .flat_map(|m| &m.voices)
        .filter(|v| v.voice_id.0 == 0)
        .flat_map(|v| &v.events)
        .filter_map(|e| match e {
            aurora_ast::Event::Note(n) => Some(n),
            _ => None,
        })
        .collect();

    assert_eq!(melody_notes.len(), 64, "expected quarter note on each beat");

    for note in &melody_notes {
        assert!(
            note.base.provenance.stage.is_some(),
            "note missing stage provenance"
        );
        assert!(
            !note.base.provenance.rule_ids.is_empty() || note.base.provenance.eval_score.is_some(),
            "note missing rule/score provenance"
        );
    }
}

#[test]
fn integration_generates_thirty_two_bars_multi_voice_with_drums() {
    let mut params = ParameterBundle::default();
    params.style.genre = "classical".into();
    params.form.section_lengths = vec![32];
    params.form.section_count = 1;
    params.mode.key = 0;
    params.mode.mode = "major".into();
    params.texture.homophony_polyphony_balance = 0.4;
    params.counterpoint.strictness = 0.6;
    params.drums.density = 0.6;
    params.search.beam_width = 8;

    let comp = generate_composition(params).expect("32-bar multi-voice generation should succeed");

    let measures: Vec<_> = comp
        .movements
        .iter()
        .flat_map(|m| &m.sections)
        .flat_map(|s| &s.phrases)
        .flat_map(|p| &p.measures)
        .collect();
    assert_eq!(measures.len(), 32);

    let voice_roles: Vec<_> = comp
        .voice_registry
        .voices
        .iter()
        .map(|v| v.role)
        .collect();
    assert!(voice_roles.contains(&aurora_ast::VoiceRole::Melody));
    assert!(voice_roles.contains(&aurora_ast::VoiceRole::Alto));
    assert!(voice_roles.contains(&aurora_ast::VoiceRole::Bass));
    assert!(voice_roles.contains(&aurora_ast::VoiceRole::Drums));

    let drums_voice = comp
        .voice_registry
        .voices
        .iter()
        .find(|v| v.role == aurora_ast::VoiceRole::Drums)
        .expect("drums voice registered");
    assert_eq!(drums_voice.midi_channel, 10);

    let melody_count = count_notes_for_voice(&measures, 0);
    let alto_count = count_notes_for_voice(&measures, 1);
    let bass_count = count_notes_for_voice(&measures, 2);
    let drum_count = count_drum_notes(&measures, 3);

    assert_eq!(melody_count, 128, "32 bars × 4 beats melody");
    assert!(alto_count >= 128, "alto should fill all rhythmic slots");
    assert!(bass_count >= 128, "bass should fill all rhythmic slots");
    assert!(drum_count > 0, "drums should have hits");

    for measure in &measures {
        assert!(!measure.harmony_slots.is_empty());
        assert!(
            measure
                .attributes
                .rehearsal_mark
                .as_deref()
                .unwrap_or("")
                .contains("RHY-"),
            "measure {} missing rhythm skeleton marker",
            measure.number.global
        );
    }

    let section = &comp.movements[0].sections[0];
    assert!(
        !section.metadata.theme_refs.is_empty(),
        "theme planning should assign theme refs"
    );
}

fn count_notes_for_voice(measures: &[&aurora_ast::Measure], voice_id: u16) -> usize {
    measures
        .iter()
        .flat_map(|m| &m.voices)
        .filter(|v| v.voice_id.0 == voice_id)
        .flat_map(|v| &v.events)
        .filter(|e| matches!(e, aurora_ast::Event::Note(_)))
        .count()
}

fn count_drum_notes(measures: &[&aurora_ast::Measure], voice_id: u16) -> usize {
    measures
        .iter()
        .flat_map(|m| &m.voices)
        .filter(|v| v.voice_id.0 == voice_id)
        .flat_map(|v| &v.events)
        .filter(|e| matches!(e, aurora_ast::Event::Note(n) if n.is_drum))
        .count()
}

#[test]
fn jazz_style_uses_jazz_progression() {
    let mut params = ParameterBundle::default();
    params.style.genre = "jazz".into();
    params.form.section_lengths = vec![4];

    let comp = generate_composition(params).expect("jazz generation");
    let first = &comp.movements[0].sections[0].phrases[0].measures[0];
    let roman = first.harmony_slots[0]
        .roman_numeral
        .as_deref()
        .unwrap_or("");
    assert!(
        roman.starts_with("ii"),
        "expected ii chord in jazz mode, got {roman}"
    );
}

#[test]
fn progress_callback_receives_stage_updates() {
    let mut params = ParameterBundle::default();
    params.form.section_lengths = vec![4];

    let stages = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let stages_clone = stages.clone();
    let callback = Box::new(move |p: aurora_engine::StageProgress| {
        stages_clone.lock().unwrap().push(p.stage_name);
    });

    aurora_engine::PipelineOrchestrator::new()
        .with_progress(callback)
        .run(&params)
        .expect("generation");

    let names = stages.lock().unwrap();
    assert!(names.iter().any(|n| n.contains("Melody")));
    assert!(names.iter().any(|n| n.contains("Drums")));
    assert!(names.iter().any(|n| n.contains("Validation")));
}
