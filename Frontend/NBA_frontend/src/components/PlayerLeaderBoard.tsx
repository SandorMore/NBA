import { useEffect, useMemo, useState } from 'react'
import { Position } from '../types/enums'

type ApiPlayer = {
  id?: number
  first_name?: string
  last_name?: string
  position?: Position
  team?: { abbreviation?: string; full_name?: string }
}

type ApiPlayerResponse = Array<Omit<ApiPlayer, 'position'> & { position?: string }> | { data?: Array<Omit<ApiPlayer, 'position'> & { position?: string }> }

function parsePosition(position?: string): Position | undefined {
  if (position === Position.G || position === Position.F || position === Position.C) return position
  return undefined
}

function getPlayers(response: ApiPlayerResponse): ApiPlayer[] {
  const players = Array.isArray(response) ? response : response.data ?? []
  return players.map((player) => ({ ...player, position: parsePosition(player.position) }))
}

export default function PlayerLeaderboard() {
  const [players, setPlayers] = useState<ApiPlayer[]>([])
  const [search, setSearch] = useState('')
  const [isLoading, setIsLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    const controller = new AbortController()

    async function loadPlayers() {
      try {
        const response = await fetch('http://127.0.0.1:8000/players', { signal: controller.signal })
        if (!response.ok) throw new Error(`Request failed with status ${response.status}`)
        const payload = await response.json() as ApiPlayerResponse
        setPlayers(getPlayers(payload))
      } catch (requestError) {
        if (requestError instanceof DOMException && requestError.name === 'AbortError') return
        setError(requestError instanceof Error ? requestError.message : 'Unable to load players.')
      } finally {
        if (!controller.signal.aborted) setIsLoading(false)
      }
    }

    void loadPlayers()
    return () => controller.abort()
  }, [])

  const filteredPlayers = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return players
    return players.filter((player) => {
      const fullName = `${player.first_name ?? ''} ${player.last_name ?? ''}`.toLowerCase()
      return fullName.includes(query) || player.team?.full_name?.toLowerCase().includes(query)
    })
  }, [players, search])

  return (
    <main className="playersPage">
      <section className="playersIntro">
        <div>
          <span className="eyebrow">League directory</span>
          <h1>Player leaderboard</h1>
          <p>Explore the league’s active roster, ranked by the order returned from NBA hub.</p>
        </div>
        <label className="playerSearch">
          <span>⌕</span>
          <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search players or teams" />
        </label>
      </section>

      <section className="leaderboard" aria-live="polite">
        <div className="leaderboardHeader"><span>Rank</span><span>Player</span><span>Team</span><span>Position</span></div>
        {isLoading && <p className="leaderboardMessage">Loading players...</p>}
        {!isLoading && error && <p className="leaderboardMessage errorMessage">{error}. Make sure the backend is running on port 8000.</p>}
        {!isLoading && !error && filteredPlayers.length === 0 && <p className="leaderboardMessage">No players match that search.</p>}
        {!isLoading && !error && filteredPlayers.map((player, index) => (
          <article className="playerRow" key={player.id ?? `${player.first_name}-${player.last_name}-${index}`}>
            <span className="playerRank">{String(index + 1).padStart(2, '0')}</span>
            <div className="playerName"><span className="playerAvatar">{player.first_name?.[0] ?? '?'}{player.last_name?.[0] ?? ''}</span><strong>{player.first_name} {player.last_name}</strong></div>
            <span className="playerTeam">{player.team?.abbreviation ?? player.team?.full_name ?? 'Free agent'}</span>
            <span className="playerPosition">{player.position || '—'}</span>
          </article>
        ))}
      </section>
    </main>
  )
}