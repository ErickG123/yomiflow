export interface MangaVolume {
  title: string
  filename: string
  filepath: string
}

export interface MangaGroup {
  title: string
  volumes: MangaVolume[]
}
