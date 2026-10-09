import { Injectable, inject, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { AppStateService } from '../../../core/services/app-state.service'
import { MangaGroup, MangaVolume } from '../models/manga.model'

const LIBRARY_FOLDER_STORAGE_KEY = 'yomiflow_manga_library_path'

@Injectable({
  providedIn: 'root',
})
export class ReaderService {
  private readonly appState = inject(AppStateService)

  readonly currentCbzPath = signal<string | null>(null)
  readonly pages = signal<string[]>([])
  readonly currentIndex = signal<number>(0)
  readonly currentPageBase64 = signal<string | null>(null)
  readonly isLoadingPage = signal<boolean>(false)

  // Library state
  readonly libraryPath = signal<string | null>(
    localStorage.getItem(LIBRARY_FOLDER_STORAGE_KEY)
  )
  readonly libraryVolumes = signal<MangaVolume[]>([])
  readonly libraryGroups = signal<MangaGroup[]>([])
  readonly isScanningLibrary = signal<boolean>(false)

  constructor() {
    const savedPath = this.libraryPath()
    if (savedPath) {
      void this.scanLibrary(savedPath)
    }
  }

  async loadCbzMetadata(path: string): Promise<string[]> {
    return invoke<string[]>('load_cbz_metadata', { path })
  }

  async getCbzPageBase64(path: string, pageName: string): Promise<string> {
    return invoke<string>('get_cbz_page_base64', { path, pageName })
  }

  async processOcr(base64Image: string): Promise<string> {
    return invoke<string>('process_ocr', { base64Image })
  }

  async scanMangaLibrary(rootPath: string): Promise<MangaVolume[]> {
    return invoke<MangaVolume[]>('scan_manga_library', { rootPath })
  }

  async openCbzFileDialog(): Promise<boolean> {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Comic Book / Manga Archive',
            extensions: ['cbz', 'zip', 'rar', 'cbr'],
          },
        ],
      })

      if (!selected) {
        return false
      }

      const filePath = typeof selected === 'string' ? selected : selected[0]
      if (!filePath) {
        return false
      }

      return await this.loadArchive(filePath)
    } catch (error) {
      console.error('Failed to open CBZ:', error)
      alert(`Erro ao abrir o ficheiro: ${error}`)
      return false
    }
  }

  async loadArchive(filePath: string): Promise<boolean> {
    this.appState.isProcessing.set(true)
    this.appState.progressMessage.set('A carregar arquivo de mangá...')

    try {
      const pagesList = await this.loadCbzMetadata(filePath)
      if (pagesList.length === 0) {
        alert('Nenhuma imagem encontrada no arquivo.')
        return false
      }

      this.currentCbzPath.set(filePath)
      this.pages.set(pagesList)
      this.currentIndex.set(0)

      await this.loadPage(0)
      return true
    } catch (error) {
      console.error('Failed to load archive:', error)
      alert(`Erro ao abrir o arquivo: ${error}`)
      return false
    } finally {
      this.appState.isProcessing.set(false)
    }
  }

  async selectLibraryFolder(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      })

      if (!selected) {
        return
      }

      const folderPath = typeof selected === 'string' ? selected : selected[0]
      if (!folderPath) {
        return
      }

      this.libraryPath.set(folderPath)
      localStorage.setItem(LIBRARY_FOLDER_STORAGE_KEY, folderPath)
      await this.scanLibrary(folderPath)
    } catch (error) {
      console.error('Failed to select manga folder:', error)
      alert(`Erro ao selecionar pasta: ${error}`)
    }
  }

  async scanLibrary(rootPath: string): Promise<void> {
    this.isScanningLibrary.set(true)
    try {
      const volumes = await this.scanMangaLibrary(rootPath)
      this.libraryVolumes.set(volumes)

      // Group volumes by title
      const groupMap = new Map<string, MangaVolume[]>()
      for (const vol of volumes) {
        const group = groupMap.get(vol.title) || []
        group.push(vol)
        groupMap.set(vol.title, group)
      }

      const groups: MangaGroup[] = Array.from(groupMap.entries()).map(([title, vols]) => ({
        title,
        volumes: vols,
      }))

      this.libraryGroups.set(groups)
    } catch (error) {
      console.error('Failed to scan manga library:', error)
    } finally {
      this.isScanningLibrary.set(false)
    }
  }

  async loadPage(index: number): Promise<void> {
    const path = this.currentCbzPath()
    const allPages = this.pages()

    if (!path || index < 0 || index >= allPages.length) {
      return
    }

    this.isLoadingPage.set(true)
    try {
      const pageName = allPages[index]
      const base64 = await this.getCbzPageBase64(path, pageName)
      this.currentIndex.set(index)
      this.currentPageBase64.set(base64)
    } catch (error) {
      console.error(`Failed to load page at index ${index}:`, error)
    } finally {
      this.isLoadingPage.set(false)
    }
  }

  // Manga reading is Right-to-Left:
  // ArrowLeft / A advances page (+1)
  async nextPage(): Promise<void> {
    if (this.currentIndex() < this.pages().length - 1) {
      await this.loadPage(this.currentIndex() + 1)
    }
  }

  // ArrowRight / D goes to previous page (-1)
  async prevPage(): Promise<void> {
    if (this.currentIndex() > 0) {
      await this.loadPage(this.currentIndex() - 1)
    }
  }

  closeReader(): void {
    this.currentCbzPath.set(null)
    this.pages.set([])
    this.currentIndex.set(0)
    this.currentPageBase64.set(null)
  }
}
