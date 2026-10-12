export type CommandPaletteAction =
  'dashboard' | 'new-note' | 'pomodoro' | 'notebook' | 'study' | 'dictionary' | 'settings'

export interface CommandPaletteCommand {
  id: string
  nameKey: string
  keywordsKey: string
  action: CommandPaletteAction
}
