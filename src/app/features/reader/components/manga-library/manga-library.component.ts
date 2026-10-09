import {
  ChangeDetectionStrategy,
  Component,
  inject,
} from '@angular/core'
import { Router } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { ReaderService } from '../../services/reader.service'
import { MangaVolume } from '../../models/manga.model'

@Component({
  selector: 'app-manga-library',
  standalone: true,
  imports: [TranslocoPipe],
  templateUrl: './manga-library.component.html',
  styleUrl: './manga-library.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MangaLibraryComponent {
  private readonly router = inject(Router)
  readonly readerService = inject(ReaderService)

  readonly libraryPath = this.readerService.libraryPath
  readonly libraryGroups = this.readerService.libraryGroups
  readonly libraryVolumes = this.readerService.libraryVolumes
  readonly isScanning = this.readerService.isScanningLibrary

  async selectFolder(): Promise<void> {
    await this.readerService.selectLibraryFolder()
  }

  async rescan(): Promise<void> {
    const path = this.libraryPath()
    if (path) {
      await this.readerService.scanLibrary(path)
    }
  }

  async openSingleFile(): Promise<void> {
    const success = await this.readerService.openCbzFileDialog()
    if (success) {
      void this.router.navigate(['/reader'])
    }
  }

  async openVolume(volume: MangaVolume): Promise<void> {
    const success = await this.readerService.loadArchive(volume.filepath)
    if (success) {
      void this.router.navigate(['/reader'])
    }
  }
}
