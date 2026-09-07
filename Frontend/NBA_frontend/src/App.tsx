import { useState } from 'react'
import { BrowserRouter, Link, Route, Routes } from 'react-router-dom'
import './App.css'

type AuthMode = 'login' | 'signin'

function Navbar({ isDark, onToggleTheme }: { isDark: boolean; onToggleTheme: () => void }) {
  return (
    <header className="navBar">
      <Link className="brand" to="/" aria-label="NBA home">
        <span>SG<span className="brandAccent">.hub</span></span>
      </Link>

      <nav className="mainNav" aria-label="Primary navigation">
        <Link className="navLink active" to="/">Home</Link>
        <Link className="navLink" to="/players">Players</Link>
        <Link className="navLink" to="/teams">Teams</Link>
      </nav>

      <div className="navActions">
        <button className="themeToggle" type="button" onClick={onToggleTheme} aria-label={`Switch to ${isDark ? 'light' : 'dark'} theme`}>
          <span aria-hidden="true">{isDark ? '☀' : '☾'}</span>
        </button>
        <Link className="authLink" to="/login">Log in</Link>
        <Link className="signInButton" to="/signin">Sign in <span aria-hidden="true">↗</span></Link>
      </div>
    </header>
  )
}

function AuthPage({ mode }: { mode: AuthMode }) {
  const isLogin = mode === 'login'

  return (
    <main className="authPage">
      <div className="authPanel">
        <span className="eyebrow">NBA hub</span>
        <h1>{isLogin ? 'Welcome back.' : 'Join the conversation.'}</h1>
        <p>{isLogin ? 'Log in to keep up with the game.' : 'Create an account for a courtside view of the league.'}</p>
        <form className="authForm">
          {!isLogin && <label>Display name<input type="text" placeholder="Your name" /></label>}
          <label>Email address<input type="email" placeholder="you@example.com" /></label>
          <label>Password<input type="password" placeholder="Enter your password" /></label>
          <button type="submit" className="formButton">{isLogin ? 'Log in' : 'Create account'} <span aria-hidden="true">↗</span></button>
        </form>
        <Link className="backLink" to="/">← Back to home</Link>
      </div>
    </main>
  )
}

function App() {
  const [isDark, setIsDark] = useState(false)

  return (
    <BrowserRouter>
      <div className={isDark ? 'app darkTheme' : 'app'}>
        <Navbar isDark={isDark} onToggleTheme={() => setIsDark((current) => !current)} />
        <Routes>
          <Route path="/login" element={<AuthPage mode="login" />} />
          <Route path="/signin" element={<AuthPage mode="signin" />} />
          <Route path="*" element={<main className="homePage"><span className="eyebrow">The game, closer</span><h1>Everything NBA.<br /><em>In one place.</em></h1><p>Follow the players, teams, and moments that make the league impossible to ignore.</p><Link className="heroButton" to="/players">Explore the league <span aria-hidden="true">↗</span></Link></main>} />
        </Routes>
      </div>
    </BrowserRouter>
  )
}

export default App
