import { useState, useEffect } from 'react'

import Summary from './components/Summary'
import type { EventObject } from './types'

import collapseIcon from './assets/collapse.svg'
import refreshIcon from './assets/refresh.svg'
import './App.css'


function App() {
  const [data, setData] = useState<EventObject[]>([]);

  const fetchData = async () => {
    const res = await fetch("http://localhost:3000/api/data");
    const json: EventObject[] = await res.json();
    setData(json);
  };

  useEffect(() => {
    fetchData();
  }, [])

  return (
    <>
      <div className="topbar">
        <span>Focus Tracker</span>
        <div className="actions">
          <img src={collapseIcon} alt="Collapse" />
          <img src={refreshIcon} onClick={fetchData} alt="Refresh" />
        </div>
      </div>
      <hr />
      <div className="chartArea">Chart</div>
      <Summary data={data} />
    </>
  )
}

export default App
