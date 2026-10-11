export interface StrokeCoordinate {
  x: number
  y: number
}

export interface SubtitleDialogue {
  id: number
  start_time: number
  end_time: number
  text: string
}

export type Subtitle = SubtitleDialogue
