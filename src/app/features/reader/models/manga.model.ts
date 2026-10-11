export interface MangaVolume {
  title: string
  filename: string
  filepath: string
}

export interface MangaGroup {
  title: string
  volumes: MangaVolume[]
}

export interface SelectionRect {
  startX: number
  startY: number
  currentX: number
  currentY: number
}
