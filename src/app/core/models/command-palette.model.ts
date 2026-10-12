export type CommandPaletteAction =
  'dashboard' | 'new-note' | 'pomodoro' | 'notebook' | 'study' | 'dictionary'

export interface CommandPaletteCommand {
  id: string
  label: string
  keywords: string[]
  action: CommandPaletteAction
}
