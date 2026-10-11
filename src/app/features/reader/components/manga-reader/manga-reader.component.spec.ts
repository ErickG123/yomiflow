import { describe, it, expect, vi, beforeEach } from 'vitest'
import { MangaReaderComponent } from './manga-reader.component'
import { Router } from '@angular/router'
import { ReaderService } from '../../services/reader.service'
import { DictionaryService } from '../../../dictionary/services/dictionary.service'
import { AudioService } from '../../../../core/services/audio.service'
import { AiModelService } from '../../services/ai-model.service'
import { EnvironmentInjector, createEnvironmentInjector, runInInjectionContext, signal } from '@angular/core'

describe('MangaReaderComponent', () => {
  let component: MangaReaderComponent
  let routerMock: { navigate: ReturnType<typeof vi.fn> }
  let readerServiceMock: Partial<ReaderService> & {
    currentCbzPath: ReturnType<typeof signal<string | null>>
    pages: ReturnType<typeof signal<string[]>>
    currentIndex: ReturnType<typeof signal<number>>
    currentPageBase64: ReturnType<typeof signal<string | null>>
    isLoadingPage: ReturnType<typeof signal<boolean>>
    nextPage: ReturnType<typeof vi.fn>
    prevPage: ReturnType<typeof vi.fn>
    openCbzFileDialog: ReturnType<typeof vi.fn>
  }
  let dictionaryServiceMock: Partial<DictionaryService> & {
    searchResults: ReturnType<typeof signal<unknown[]>>
    kanjiSearchResults: ReturnType<typeof signal<unknown[]>>
    isLoading: ReturnType<typeof signal<boolean>>
    search: ReturnType<typeof vi.fn>
    addToSrs: ReturnType<typeof vi.fn>
    removeFromSrs: ReturnType<typeof vi.fn>
  }
  let audioServiceMock: { speak: ReturnType<typeof vi.fn> }
  let aiServiceMock: Partial<AiModelService> & {
    isDownloading: ReturnType<typeof signal<boolean>>
    downloadProgress: ReturnType<typeof signal<number>>
    isModelReady: ReturnType<typeof signal<boolean>>
    ensureModelExists: ReturnType<typeof vi.fn>
  }

  beforeEach(() => {
    vi.restoreAllMocks()

    routerMock = {
      navigate: vi.fn(),
    }

    readerServiceMock = {
      currentCbzPath: signal<string | null>('C:/manga/test_vol1.cbz'),
      pages: signal<string[]>(['p01.jpg', 'p02.jpg', 'p03.jpg']),
      currentIndex: signal<number>(0),
      currentPageBase64: signal<string | null>('base64_img_data'),
      isLoadingPage: signal<boolean>(false),
      nextPage: vi.fn(),
      prevPage: vi.fn(),
      openCbzFileDialog: vi.fn(),
    }

    dictionaryServiceMock = {
      searchResults: signal([]),
      kanjiSearchResults: signal([]),
      isLoading: signal(false),
      search: vi.fn(),
      addToSrs: vi.fn(),
      removeFromSrs: vi.fn(),
    }

    audioServiceMock = {
      speak: vi.fn(),
    }

    aiServiceMock = {
      isDownloading: signal(false),
      downloadProgress: signal(0),
      isModelReady: signal(true),
      ensureModelExists: vi.fn().mockResolvedValue(true),
    }

    const providers = [
      { provide: Router, useValue: routerMock },
      { provide: ReaderService, useValue: readerServiceMock },
      { provide: DictionaryService, useValue: dictionaryServiceMock },
      { provide: AudioService, useValue: audioServiceMock },
      { provide: AiModelService, useValue: aiServiceMock },
    ]

    const parentInjector = { get: (token: unknown) => providers.find((p) => p.provide === token)?.useValue ?? null } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new MangaReaderComponent()
    })
  })

  it('should redirect to library on init if no manga is currently opened', () => {
    readerServiceMock.currentCbzPath.set(null)
    component.ngOnInit()
    expect(routerMock.navigate).toHaveBeenCalledWith(['/manga'])
  })

  it('should navigate next page when pressing ArrowLeft or "a" (Japanese manga reading direction)', () => {
    const eventArrow = new KeyboardEvent('keydown', { key: 'ArrowLeft' })
    component.handleKeyboardEvent(eventArrow)
    expect(readerServiceMock.nextPage).toHaveBeenCalledTimes(1)

    const eventA = new KeyboardEvent('keydown', { key: 'a' })
    component.handleKeyboardEvent(eventA)
    expect(readerServiceMock.nextPage).toHaveBeenCalledTimes(2)
  })

  it('should navigate previous page when pressing ArrowRight or "d"', () => {
    const eventArrow = new KeyboardEvent('keydown', { key: 'ArrowRight' })
    component.handleKeyboardEvent(eventArrow)
    expect(readerServiceMock.prevPage).toHaveBeenCalledTimes(1)

    const eventD = new KeyboardEvent('keydown', { key: 'd' })
    component.handleKeyboardEvent(eventD)
    expect(readerServiceMock.prevPage).toHaveBeenCalledTimes(2)
  })

  it('should ignore hotkeys if the user is typing inside an input or textarea', () => {
    const mockInput = document.createElement('input')
    document.body.appendChild(mockInput)
    mockInput.focus()

    const event = new KeyboardEvent('keydown', { key: 'a' })
    component.handleKeyboardEvent(event)

    expect(readerServiceMock.nextPage).not.toHaveBeenCalled()
    document.body.removeChild(mockInput)
  })

  it('should close the OCR drawer on Escape without navigating back to library', () => {
    component.isDrawerOpen.set(true)

    const eventEsc = new KeyboardEvent('keydown', { key: 'Escape' })
    component.handleKeyboardEvent(eventEsc)

    expect(component.isDrawerOpen()).toBe(false)
    expect(routerMock.navigate).not.toHaveBeenCalled()
  })

  it('should navigate back to library on Escape if drawer is already closed', () => {
    component.isDrawerOpen.set(false)

    const eventEsc = new KeyboardEvent('keydown', { key: 'Escape' })
    component.handleKeyboardEvent(eventEsc)

    expect(routerMock.navigate).toHaveBeenCalledWith(['/manga'])
  })
})
