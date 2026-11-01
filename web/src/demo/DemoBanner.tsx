/** The line across the top of the online demo: what it is, and where the real app is. */
export function DemoBanner() {
  return (
    <div role="note" className="border-b border-line bg-raised px-4 py-2 text-center text-[13.5px] leading-snug text-muted">
      <strong className="font-semibold text-ink">Demo with a made-up library.</strong> Picks are written ahead of time; the
      real app asks your own AI. What you change stays in this tab and is gone when you reload.{' '}
      <a href="https://upnext.games" className="whitespace-nowrap text-ink underline underline-offset-2">
        Get UpNext
      </a>
    </div>
  )
}
