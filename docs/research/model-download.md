# Model download and Windows installer

## The model file

- Source: `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin`, which redirects (302) to Hugging Face's CDN.
- Size 1,624,555,275 bytes, SHA-256 `1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69`, from the Hugging Face API (`/api/models/ggerganov/whisper.cpp/tree/main`, `lfs.oid`) and the `X-Linked-Size` and `X-Linked-ETag` headers.
- The copy the app has used since October (`%APPDATA%\Podcast-Monitor\models\ggml-large-v3-turbo.bin`) has the same size and SHA-256 (`sha256sum`), so the pinned hash is the one every measurement so far ran on.

## Chosen by Felipe

- The window shows a notice and a Descargar modelo button when the model is missing; runs stay off until it is there.
- The finished file is checked against the pinned SHA-256; a wrong file is deleted and the user is told to retry.
- An interrupted download resumes from the partial file with an HTTP range request.
- Wording:
  - Notice: "Para transcribir, la app necesita el modelo de voz (1,6 GB). Se descarga una sola vez."
  - Button: "Descargar modelo"
  - Progress: "Descargando el modelo de voz: 640 de 1.549 MB (41 %)"
  - Check: "Revisando que el modelo esté completo…"
  - Done: "Modelo de voz listo."
  - Interrupted: "La descarga se interrumpió. Revisa la conexión a internet y usa Descargar modelo para seguir donde quedó."
  - Bad file: "El modelo descargado llegó dañado y se borró. Usa Descargar modelo para bajarlo de nuevo."
  - Run without model: "Falta el modelo de voz. Usa Descargar modelo y luego Continuar."
- Windows is installed with Inno Setup (exe, Start menu shortcut, uninstaller); the model stays an in-app download, so the Mac build uses the same path.

## Inno Setup

- Installed here: Inno Setup 6 at `C:\Program Files (x86)\Inno Setup 6` (`ISCC.exe`, examples `DownloadFiles.iss` and `CodeDownloadFiles.iss`).
- Licence: use for any purpose, commercial included; copyright notices kept in source and binary redistributions (https://jrsoftware.org/files/is/license.txt). Source public.
- Its download page takes the expected SHA-256 as the third argument of `DownloadPage.Add` and stops with "Invalid file hash" on a mismatch (https://jrsoftware.org/ishelp/topic_isxfunc_createdownloadpage.htm, https://groups.google.com/g/innosetup/c/ZJcIBLvMDuI). No resume of a partial download found in its docs. Windows only.
- Open-source alternatives looked at: NSIS (zlib licence; downloads and hashes through plugins) and WiX (MS-RL; MSI, no built-in download page).
