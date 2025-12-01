//
// Copyright (c) 2025 Nathan Fiedler
//
import {
  createMemo,
  createSignal,
  Index,
  type JSX,
  type Setter,
  Show,
} from 'solid-js'
import { action, redirect, useAction, useSubmission } from '@solidjs/router'

function Upload() {
  const datefmt = new Intl.DateTimeFormat()
  const [selectedFiles, setSelectedFiles] = createSignal<Array<File>>([])
  const [droppedFiles, setDroppedFiles] = createSignal<Array<File>>([])
  const filesSelected: JSX.EventHandlerWithOptionsUnion<
    HTMLInputElement,
    Event,
    JSX.ChangeEventHandler<HTMLInputElement, Event>
  > = (event) => {
    // event.target.files is a FileList, not an Array
    setSelectedFiles(Array.from<File>(event.target.files!))
  }
  const hasFiles = createMemo(() => {
    return selectedFiles().length > 0 || droppedFiles().length > 0
  })
  // indicates number of files uploaded so far as percentage
  const [progress, setProgress] = createSignal(0)
  const startUploadAction = action(async (): Promise<void> => {
    const allFiles = selectedFiles().concat(droppedFiles())
    await uploadFiles(allFiles, setProgress)
    throw redirect('/pending')
  })
  const startUpload = useAction(startUploadAction)
  const uploadSubmission = useSubmission(startUploadAction)

  return (
    <>
      <div class="container">
        <nav class="level">
          <div class="level-left">
            <div class="level-item">
              <button class="button" disabled>
                Import
              </button>
              <div class="block ml-2">
                <p class="subtitle is-5">
                  from the <code>uploads</code>directory
                </p>
              </div>
            </div>
          </div>

          <div class="level-right">
            <div class="level-item">
              <div class="card">
                <div class="field is-grouped">
                  <div class="file">
                    <label class="file-label">
                      <input
                        class="file-input"
                        type="file"
                        multiple
                        name="uploads"
                        on:change={filesSelected}
                        disabled={uploadSubmission.pending}
                      />
                      <span class="file-cta">
                        <span class="file-icon">
                          <i class="fas fa-upload"></i>
                        </span>
                        <span class="file-label">Choose Files</span>
                      </span>
                    </label>
                  </div>
                </div>
              </div>
            </div>
            <div class="level-item">
              <button
                class="button"
                class:is-loading={uploadSubmission.pending}
                onClick={() => startUpload()}
                disabled={!hasFiles() || uploadSubmission.pending}
              >
                Start Upload
              </button>
            </div>
          </div>
        </nav>
        <Show
          when={uploadSubmission.pending}
          fallback={<progress class="mt-4 progress" value="0" max="100" />}
        >
          <progress class="mt-4 progress" value={progress()} max="100">
            {progress().toString()}
          </progress>
        </Show>
      </div>

      <DropZone setDroppedFiles={setDroppedFiles}>
        <Show when={hasFiles()} fallback={<></>}>
          <table class="table is-fullwidth">
            <thead>
              <tr>
                <th>File</th>
                <th>Type</th>
                <th>Size</th>
                <th>Date</th>
              </tr>
            </thead>
            <tbody>
              <Index each={selectedFiles()}>
                {(file) => (
                  <tr>
                    <td>{file().name}</td>
                    <td>{file().type}</td>
                    <td>{file().size}</td>
                    <td>{datefmt.format(file().lastModified)}</td>
                  </tr>
                )}
              </Index>
              <Index each={droppedFiles()}>
                {(file) => (
                  <tr>
                    <td>{file().name}</td>
                    <td>{file().type}</td>
                    <td>{file().size}</td>
                    <td>{datefmt.format(file().lastModified)}</td>
                  </tr>
                )}
              </Index>
            </tbody>
            <tfoot>
              <tr>
                <th>File</th>
                <th>Type</th>
                <th>Size</th>
                <th>Date</th>
              </tr>
            </tfoot>
          </table>
        </Show>
      </DropZone>
    </>
  )
}

interface DropZoneProps {
  setDroppedFiles: Setter<Array<File>>
  children: any
}

function DropZone(props: DropZoneProps) {
  const [isDragOver, setIsDragOver] = createSignal(false)

  const handleDragOver: JSX.EventHandler<HTMLDivElement, DragEvent> = (
    event
  ) => {
    event.preventDefault()
    setIsDragOver(true)
  }

  const handleDragLeave = () => {
    setIsDragOver(false)
  }

  const handleDrop: JSX.EventHandler<HTMLDivElement, DragEvent> = (event) => {
    event.preventDefault()
    setIsDragOver(false)
    props.setDroppedFiles(Array.from(event.dataTransfer?.files ?? []))
  }

  return (
    <section class="section">
      <p>You can drop files into the drop zone below.</p>
      <div
        class="content"
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
        style={{
          'border-style': 'dashed',
          'min-height': '14em',
          'border-color': isDragOver() ? 'green' : 'white',
        }}
      >
        {props.children}
      </div>
    </section>
  )
}

async function uploadFiles(
  selectedFiles: Array<File>,
  setProgress: Setter<number>
) {
  for (const [index, file] of selectedFiles.entries()) {
    const formData = new FormData()
    // appending a File will include its filename
    formData.append('file_blob', file)
    formData.append('last_modified', file.lastModified.toString())
    try {
      await fetch('/assets/upload', {
        method: 'POST',
        body: formData,
      })
    } catch (error) {
      console.error('Error uploading files:', error)
    }
    setProgress(((index + 1) * 100) / selectedFiles.length)
  }
}

export default Upload
