import collapseIcon from './assets/collapse.svg'
import refreshIcon from './assets/refresh.svg'
import './App.css'

function App() {
  return (
    <>
      <div className="topbar">
        <span>Focus Tracker</span>
        <div className="actions">
          <img src={collapseIcon} alt="Collapse" />
          <img src={refreshIcon} alt="Refresh" />
        </div>
      </div>
    </>
  )
}

export default App
