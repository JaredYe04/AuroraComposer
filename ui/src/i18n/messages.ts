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
  | 'params.style'
  | 'params.bars'
  | 'params.beamWidth'
  | 'params.emotion'
  | 'params.valence'
  | 'params.arousal'
  | 'params.harmony'
  | 'params.complexity'
  | 'params.counterpoint'
  | 'params.strictness'
  | 'params.drums'
  | 'params.density'
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
  | 'plugins.title';

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
    'params.style': 'Style',
    'params.bars': 'Bars',
    'params.beamWidth': 'Beam width',
    'params.emotion': 'Emotion',
    'params.valence': 'Valence',
    'params.arousal': 'Arousal',
    'params.harmony': 'Harmony',
    'params.complexity': 'Complexity',
    'params.counterpoint': 'Counterpoint',
    'params.strictness': 'Strictness',
    'params.drums': 'Drums',
    'params.density': 'Density',
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
    'params.style': '风格',
    'params.bars': '小节数',
    'params.beamWidth': '束宽',
    'params.emotion': '情绪',
    'params.valence': '效价',
    'params.arousal': '唤醒度',
    'params.harmony': '和声',
    'params.complexity': '复杂度',
    'params.counterpoint': '对位',
    'params.strictness': '严格度',
    'params.drums': '鼓组',
    'params.density': '密度',
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
  },
};
