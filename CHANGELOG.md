# Changelog

## 0.5.2

- Fix multilingual Wayland window titles with system font fallback.
- Show only the filename in the model window title; move triangle count and load time to **File → Info**.
- Move model geometry details from About to Info.
- Replace the scene context menu with a standard popup that stays within the window.
- Include all View menu options in the scene context menu.
- Add Fontconfig and FreeType dependencies to Linux packages and build setup.

## 0.5.1

- Allow 3MF geometry to load when optional thumbnail files are missing.
- Enable system clipboard integration for text copy and paste.
- Add a **Copy error** button to the error dialog.
- Prevent Ctrl+R and Ctrl+W from triggering model reset and wireframe shortcuts, and respect keyboard events consumed by the UI.

## 0.5.0

- Add OBJ geometry loading, including indexed faces, negative indices and triangle-fan triangulation.
- Add 3MF triangle-mesh loading with build items, nested components, repeated instances, transforms and cross-file component references.
- Convert 3MF dimensions to millimeters and reject mixed units across model parts.
- Support OBJ and 3MF in file dialogs, drag and drop, CLI information and screenshot export.
- Add Linux MIME associations for OBJ and 3MF.
- Retain uniform model coloring; materials, textures and slicer settings are not displayed.
