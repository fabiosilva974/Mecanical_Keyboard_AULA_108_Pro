# AULA F108 Pro GUI - Internationalization (i18n) Guide

## 1. Architecture
The localization engine is 100% data-driven and requires **no code recompilation** to add new languages.

```
ui/locales/
├── en.json      # English (Default Reference)
├── pt_BR.json   # Portuguese (Brazil)
└── es_ES.json   # Spanish (Example community translation)
```

## 2. Adding a New Language

1. Copy `ui/locales/en.json` to `ui/locales/<locale_code>.json` (e.g. `de_DE.json`, `fr_FR.json`).
2. Translate the values while preserving JSON keys:
   ```json
   {
     "nav": {
       "home": "Startseite",
       "remap": "Tastenbelegung",
       "lighting": "Lichteffekte"
     }
   }
   ```
3. Add the new option to the language dropdown in `ui/index.html`:
   ```html
   <option value="de_DE">Deutsch (DE)</option>
   ```

## 3. Dynamic Tagging Reference
- `data-i18n="section.key"`: Replaces element's `textContent`.
- `data-i18n-placeholder="section.key"`: Replaces `<input>` or `<textarea>` placeholder.
- `data-i18n-title="section.key"`: Replaces tooltip `title` attribute.
