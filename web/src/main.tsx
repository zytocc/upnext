import { StrictMode, type ReactNode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import { WindowFrame } from './components/WindowFrame'
import { markFrame } from './lib/frame'
import { reloadOnStaleScripts } from './lib/reload'
import { ErrorBoundary } from './components/ErrorBoundary'

markFrame()
reloadOnStaleScripts()

function render(banner: ReactNode = null) {
  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      {banner}
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
      <WindowFrame />
    </StrictMode>,
  )
}

// The online demo (`npm run build:demo`) answers from a recording in the
// page; every other build leaves this out.
if (import.meta.env.MODE === 'demo') {
  void Promise.all([import('./demo/install'), import('./demo/DemoBanner')]).then(async ([{ install }, { DemoBanner }]) => {
    await install()
    render(<DemoBanner />)
  })
} else {
  render()
}
