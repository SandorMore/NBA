import { useEffect, useMemo, useState } from 'react'
type ApiPlayer = {
  id: number
  name: string
  rank: number
  ppg: number
  rpg: number
  apg: number
  bpm: number
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
        const response = await fetch('http://127.0.0.1:8000/leaderboard?season=2024&limit=100', { signal: controller.signal })
        if (!response.ok) throw new Error(`Request failed with status ${response.status}`)
        const payload = await response.json() as ApiPlayer[]
        setPlayers(payload)
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
      return player.name.toLowerCase().includes(query)
    })
  }, [players, search])

  return (
    <main className="playersPage">
      <section className="playersIntro">
        <div>
          <span className="eyebrow">League directory</span>
          <h1>Player leaderboard</h1>
          <p>Explore the 2024 season leaders ranked by box plus/minus.</p>
        </div>
        <label className="playerSearch">
          <span>⌕</span>
          <input value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Search players or teams" />
        </label>
      </section>

      <section className="leaderboard" aria-live="polite">
        <div className="leaderboardHeader"><span>Rank</span><span>Player</span><span>PPG</span><span>RPG</span><span>APG</span><span>BPM</span></div>
        {isLoading && <p className="leaderboardMessage">Loading players...</p>}
        {!isLoading && error && <p className="leaderboardMessage errorMessage">{error}. Make sure the backend is running on port 8000.</p>}
        {!isLoading && !error && filteredPlayers.length === 0 && <p className="leaderboardMessage">No leaderboard data found. Run <code>cargo run -- sync 2024</code> from the NBA backend folder first.</p>}
        {!isLoading && !error && filteredPlayers.map((player, index) => (
          <article className="playerRow" key={player.id}>
            <span className="playerRank">{String(player.rank || index + 1).padStart(2, '0')}</span>
            <div className="playerName"><span className="playerAvatar">{player.name.split(' ').map((part) => part[0]).join('').slice(0, 2)}</span><strong>{player.name}</strong></div>
            <span className="playerStat">{player.ppg.toFixed(1)}</span>
            <span className="playerStat">{player.rpg.toFixed(1)}</span>
            <span className="playerStat">{player.apg.toFixed(1)}</span>
            <span className="playerStat playerBpm">{player.bpm > 0 ? '+' : ''}{player.bpm.toFixed(1)}</span>
          </article>
        ))}
      </section>
    </main>
  )
}