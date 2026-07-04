export type MessageKey =
  | 'app.title'
  | 'app.badge'
  | 'settings.theme'
  | 'settings.language'
  | 'settings.themeDark'
  | 'settings.themeLight'
  | 'settings.langEn'
  | 'settings.langZh'
  | 'project.new'
  | 'project.load'
  | 'project.save'
  | 'params.title'
  | 'params.styleForm'
  | 'params.key'
  | 'params.mode'
  | 'params.style'
  | 'params.bars'
  | 'params.beamWidth'
  | 'params.random'
  | 'params.seed'
  | 'params.rollSeed'
  | 'params.emotion'
  | 'params.valence'
  | 'params.arousal'
  | 'params.harmony'
  | 'params.complexity'
  | 'params.progression'
  | 'params.progressionLoop'
  | 'params.progressionFlow'
  | 'params.counterpoint'
  | 'params.strictness'
  | 'params.drums'
  | 'params.density'
  | 'params.accentEmphasis'
  | 'params.hihatDensity'
  | 'params.tonalConservatism'
  | 'params.keyHint'
  | 'params.modeHint'
  | 'params.styleHint'
  | 'params.barsHint'
  | 'params.beamWidthHint'
  | 'params.seedHint'
  | 'params.valenceHint'
  | 'params.arousalHint'
  | 'params.complexityHint'
  | 'params.progressionHint'
  | 'params.tonalConservatismHint'
  | 'params.accompanimentHint'
  | 'params.strictnessHint'
  | 'params.densityHint'
  | 'params.accentEmphasisHint'
  | 'params.hihatDensityHint'
  | 'params.accompaniment'
  | 'params.accompanimentAuto'
  | 'params.accompanimentPiano'
  | 'params.accompanimentStrings'
  | 'params.generate'
  | 'params.generating'
  | 'summary.bars'
  | 'summary.notes'
  | 'summary.measure'
  | 'playback.title'
  | 'playback.play'
  | 'playback.stop'
  | 'playback.volume'
  | 'playback.hint'
  | 'playback.melodyEngine'
  | 'playback.melodyGm'
  | 'playback.melodySine'
  | 'playback.melodySquare'
  | 'playback.melodyTriangle'
  | 'playback.melodySawtooth'
  | 'score.title'
  | 'score.refresh'
  | 'score.loading'
  | 'score.empty'
  | 'score.tabAbc'
  | 'score.tabMusicXml'
  | 'score.tabPdf'
  | 'score.downloadPdf'
  | 'export.title'
  | 'export.downloadMidi'
  | 'export.downloadMusicXml'
  | 'export.downloadAbc'
  | 'inspector.title'
  | 'inspector.empty'
  | 'inspector.loading'
  | 'piano.empty'
  | 'piano.clickHint'
  | 'plugins.title'
  | 'plugins.menu'
  | 'plugins.hint'
  | 'plugins.register'
  | 'plugins.refresh'
  | 'plugins.registered'
  | 'plugins.empty'
  | 'tools.pointer'
  | 'tools.box'
  | 'tools.brush'
  | 'tools.eraser'
  | 'tools.split'
  | 'voices.title'
  | 'voices.all'
  | 'editor.pianoRoll'
  | 'editor.playlist'
  | 'score.zoomIn'
  | 'score.zoomOut'
  | 'score.zoomFit'
  | 'score.panHint'
  | 'score.pagePrev'
  | 'score.pageNext'
  | 'playlist.patterns'
  | 'playlist.rowLabel'
  | 'playlist.randomColor'
  | 'playlist.duplicatePattern'
  | 'playlist.addPattern'
  | 'playlist.deletePattern';

export type Messages = Record<MessageKey, string>;

export const messages: Record<'en' | 'zh', Messages> = {
  en: {
    'app.title': 'Aurora Composer',
    'app.badge': 'Phase 3',
    'settings.theme': 'Theme',
    'settings.language': 'Language',
    'settings.themeDark': 'Dark',
    'settings.themeLight': 'Light',
    'settings.langEn': 'English',
    'settings.langZh': '中文',
    'project.new': 'New',
    'project.load': 'Load',
    'project.save': 'Save',
    'params.title': 'Parameters',
    'params.styleForm': 'Style & Form',
    'params.key': 'Key',
    'params.mode': 'Mode',
    'params.style': 'Style',
    'params.bars': 'Bars',
    'params.beamWidth': 'Beam width',
    'params.random': 'Random',
    'params.seed': 'Seed',
    'params.rollSeed': 'Roll random seed',
    'params.emotion': 'Emotion',
    'params.valence': 'Valence',
    'params.arousal': 'Arousal',
    'params.harmony': 'Harmony',
    'params.complexity': 'Complexity',
    'params.progression': 'Progression',
    'params.progressionLoop': 'Loop (repeating cell)',
    'params.progressionFlow': 'Flow (phrase arc)',
    'params.counterpoint': 'Counterpoint',
    'params.strictness': 'Strictness',
    'params.drums': 'Drums',
    'params.density': 'Density',
    'params.accentEmphasis': 'Accent emphasis',
    'params.hihatDensity': 'Hi-hat density',
    'params.tonalConservatism': 'Tonal conservatism',
    'params.keyHint': 'Root pitch of the scale. Does not change motif shape by itself.',
    'params.modeHint': 'Scale collection (major, minor, modes). Affects harmony templates and diatonic melody pool.',
    'params.styleHint': 'High level preset. Jazz adds syncopation and chromatic harmony; classical favors cadences and counterpoint.',
    'params.barsHint': 'Length of the piece in measures. Longer forms allow more phrase repetition and development.',
    'params.beamWidthHint': 'Higher: more melodic paths explored, slower but often smoother. Lower: faster, may sound repetitive or get stuck.',
    'params.seedHint': 'Controls motif pattern, phrase theme assignment, and tie-breaks. Same seed = same result; change seed for a new melody.',
    'params.valenceHint': 'Higher: brighter major bias and simpler harmony. Lower: darker color and more minor/borrowed chords.',
    'params.arousalHint': 'Higher: faster tempo and denser rhythm. Lower: slower, more spacious phrasing.',
    'params.complexityHint': 'Higher: extensions, secondary dominants, borrowed chords. Lower: simple triads. High + low tonal conservatism may sound harsh (auto-clamped).',
    'params.progressionHint': 'Loop: repeating chord cell (pop). Flow: non-repeating harmonic arc with cadence toward the end.',
    'params.tonalConservatismHint': 'Higher: chord tones on strong beats, diatonic melody, fewer clashes. Lower: more chromatic passing tones (minimum enforced for stability).',
    'params.accompanimentHint': 'Block chord texture instrument. Auto follows style preset.',
    'params.strictnessHint': 'Higher: counterpoint rules active, smoother inner voices. Lower: counterpoint stage may be skipped.',
    'params.densityHint': 'Drum fill density. Does not affect melody harmony.',
    'params.accentEmphasisHint': 'Strong-beat kick/snare weight in the drum pattern.',
    'params.hihatDensityHint': 'How often hi-hats appear between beats.',
    'params.accompaniment': 'Chord accompaniment',
    'params.accompanimentAuto': 'Auto (style)',
    'params.accompanimentPiano': 'Piano',
    'params.accompanimentStrings': 'Strings',
    'params.generate': 'Generate',
    'params.generating': 'Generating…',
    'summary.bars': 'bars',
    'summary.notes': 'notes',
    'summary.measure': 'Measure',
    'playback.title': 'Playback',
    'playback.play': 'Play',
    'playback.stop': 'Stop',
    'playback.volume': 'Vol',
    'playback.hint': 'Generate a composition to enable playback.',
    'playback.melodyEngine': 'Melody',
    'playback.melodyGm': 'Piano (GM)',
    'playback.melodySine': 'Sine',
    'playback.melodySquare': 'Square',
    'playback.melodyTriangle': 'Triangle',
    'playback.melodySawtooth': 'Sawtooth',
    'score.title': 'Score',
    'score.refresh': 'Refresh',
    'score.loading': 'Loading…',
    'score.empty': 'Generate a composition to view the score.',
    'score.tabAbc': 'ABC',
    'score.tabMusicXml': 'MusicXML',
    'score.tabPdf': 'PDF',
    'score.downloadPdf': 'Download PDF',
    'export.title': 'Export',
    'export.downloadMidi': 'Download MIDI',
    'export.downloadMusicXml': 'Download MusicXML',
    'export.downloadAbc': 'Download ABC',
    'inspector.title': 'Inspector',
    'inspector.empty': 'Select a note to inspect provenance',
    'inspector.loading': 'Loading provenance…',
    'piano.empty': 'No notes to display.',
    'piano.clickHint': 'Click for full provenance',
    'plugins.title': 'Plugins',
    'plugins.menu': 'Plugins',
    'plugins.hint':
      'Built-in style plugins (classical, jazz, pop) and AI stub are always active. Register external WASM plugins below.',
    'plugins.register': 'Register WASM Plugin…',
    'plugins.refresh': 'Refresh',
    'plugins.registered': 'Plugin registered.',
    'plugins.empty': 'No external WASM plugins discovered.',
    'tools.pointer': 'Pointer',
    'tools.box': 'Box select',
    'tools.brush': 'Brush',
    'tools.eraser': 'Eraser',
    'tools.split': 'Split',
    'voices.title': 'Voice',
    'voices.all': 'All',
    'editor.pianoRoll': 'Piano roll',
    'editor.playlist': 'Playlist',
    'score.zoomIn': 'Zoom in',
    'score.zoomOut': 'Zoom out',
    'score.zoomFit': 'Fit to view',
    'score.panHint': 'Drag to pan · Ctrl+wheel to zoom',
    'score.pagePrev': 'Previous page',
    'score.pageNext': 'Next page',
    'playlist.patterns': 'Patterns',
    'playlist.rowLabel': 'Arrangement',
    'playlist.randomColor': 'Random color',
    'playlist.duplicatePattern': 'Duplicate pattern',
    'playlist.addPattern': 'New pattern',
    'playlist.deletePattern': 'Delete pattern',
  },
  zh: {
    'app.title': 'Aurora Composer',
    'app.badge': 'Phase 3',
    'settings.theme': '主题',
    'settings.language': '语言',
    'settings.themeDark': '深色',
    'settings.themeLight': '浅色',
    'settings.langEn': 'English',
    'settings.langZh': '中文',
    'project.new': '新建',
    'project.load': '打开',
    'project.save': '保存',
    'params.title': '参数',
    'params.styleForm': '风格与结构',
    'params.key': '调性',
    'params.mode': '调式',
    'params.style': '风格',
    'params.bars': '小节数',
    'params.beamWidth': '束宽',
    'params.random': '随机',
    'params.seed': '随机种子',
    'params.rollSeed': '掷骰子',
    'params.emotion': '情绪',
    'params.valence': '效价',
    'params.arousal': '唤醒度',
    'params.harmony': '和声',
    'params.complexity': '复杂度',
    'params.progression': '和弦进行',
    'params.progressionLoop': '循环 (Loop)',
    'params.progressionFlow': '流动 (Flow)',
    'params.counterpoint': '对位',
    'params.strictness': '严格度',
    'params.drums': '鼓组',
    'params.density': '密度',
    'params.accentEmphasis': '重拍强调',
    'params.hihatDensity': 'Hi-hat 密度',
    'params.tonalConservatism': '调性保守度',
    'params.keyHint': '音阶主音。单独改变调性不会直接改变动机轮廓。',
    'params.modeHint': '音阶类型（大调、小调、教会调式）。影响和声模板与旋律调内音集合。',
    'params.styleHint': '风格预设。爵士增加切分与半音和声；古典强调终止式与对位。',
    'params.barsHint': '乐曲长度（小节）。更长便于动机重复与发展。',
    'params.beamWidthHint': '较高：探索更多旋律路径，更慢但通常更流畅。较低：更快，可能重复或搜索失败。',
    'params.seedHint': '控制动机型、短语主题分配与搜索平局打破。相同种子=相同结果；换种子得新旋律。',
    'params.valenceHint': '较高：偏明亮大调、和声较简单。较低：偏暗、小调/借用和弦更多。',
    'params.arousalHint': '较高：速度更快、节奏更密。较低：更慢、留白更多。',
    'params.complexityHint': '较高：扩展和弦、副属和弦、借用和弦。较低：简单三和弦。高复杂度+低调性可能刺耳（会自动限制）。',
    'params.progressionHint': 'Loop：重复和弦单元（流行）。Flow：不重复的和声弧线，句末收束。',
    'params.tonalConservatismHint': '较高：强拍和弦音、调内旋律、更少冲突。较低：更多半音经过音（有最低保护避免乱弹）。',
    'params.accompanimentHint': '块状和弦伴奏音色。Auto 跟随风格预设。',
    'params.strictnessHint': '较高：启用对位规则，内声部更平滑。较低：可能跳过对位阶段。',
    'params.densityHint': '鼓组填充密度。不影响旋律与和声。',
    'params.accentEmphasisHint': 'Kick/Snare 在强拍上的力度。',
    'params.hihatDensityHint': 'Hi-hat 在拍间的出现频率。',
    'params.accompaniment': '和弦伴奏',
    'params.accompanimentAuto': '自动（随风格）',
    'params.accompanimentPiano': '钢琴',
    'params.accompanimentStrings': '弦乐',
    'params.generate': '生成',
    'params.generating': '生成中…',
    'summary.bars': '小节',
    'summary.notes': '音符',
    'summary.measure': '小节',
    'playback.title': '播放',
    'playback.play': '播放',
    'playback.stop': '停止',
    'playback.volume': '音量',
    'playback.hint': '请先生成作品以启用播放。',
    'playback.melodyEngine': '旋律',
    'playback.melodyGm': '钢琴 (GM)',
    'playback.melodySine': '正弦波',
    'playback.melodySquare': '方波',
    'playback.melodyTriangle': '三角波',
    'playback.melodySawtooth': '锯齿波',
    'score.title': '曲谱',
    'score.refresh': '刷新',
    'score.loading': '加载中…',
    'score.empty': '请先生成作品以查看曲谱。',
    'score.tabAbc': 'ABC',
    'score.tabMusicXml': 'MusicXML',
    'score.tabPdf': 'PDF',
    'score.downloadPdf': '下载 PDF',
    'export.title': '导出',
    'export.downloadMidi': '下载 MIDI',
    'export.downloadMusicXml': '下载 MusicXML',
    'export.downloadAbc': '下载 ABC',
    'inspector.title': '检查器',
    'inspector.empty': '选择音符以查看溯源',
    'inspector.loading': '加载溯源中…',
    'piano.empty': '暂无音符。',
    'piano.clickHint': '点击查看完整溯源',
    'plugins.title': '插件',
    'plugins.menu': '插件',
    'plugins.hint':
      '内置风格插件（古典、爵士、流行）和 AI 占位始终启用。在下方注册外部 WASM 插件。',
    'plugins.register': '注册 WASM 插件…',
    'plugins.refresh': '刷新',
    'plugins.registered': '插件已注册。',
    'plugins.empty': '未发现外部 WASM 插件。',
    'tools.pointer': '指针',
    'tools.box': '框选',
    'tools.brush': '画笔',
    'tools.eraser': '橡皮',
    'tools.split': '分割',
    'voices.title': '声部',
    'voices.all': '全部',
    'editor.pianoRoll': '钢琴窗',
    'editor.playlist': '播放列表',
    'score.zoomIn': '放大',
    'score.zoomOut': '缩小',
    'score.zoomFit': '适应窗口',
    'score.panHint': '拖拽平移 · Ctrl+滚轮缩放',
    'score.pagePrev': '上一页',
    'score.pageNext': '下一页',
    'playlist.patterns': 'Pattern',
    'playlist.rowLabel': '编排',
    'playlist.randomColor': '随机颜色',
    'playlist.duplicatePattern': '复制 Pattern',
    'playlist.addPattern': '新建 Pattern',
    'playlist.deletePattern': '删除 Pattern',
  },
};
