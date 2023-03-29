import { useState } from 'react'
import * as tb from "timeblok-js"


function App() {
  const [leftText, setLeftText] = useState('Left Text')
  const [rightText, setRightText] = useState('Right Text')
  const handleClick = () => {
    setRightText("Compiling...")
    let timeout = setTimeout(() => {
      setRightText('Compilation took too long. An exception might have occured, please wait until we further improve our error handling process.')
    }, 3000)
    let s = tb.compile(leftText, BigInt(Date.now()))
    clearTimeout(timeout)
    setRightText("Done.")
    if (typeof s === 'string') {
      setRightText(s)
    } else {
      setRightText('error')
    } 
  }
  return (
    <div className="container mx-auto w-screen">
      <h1>Timeblok Playground! (Very early alpha mode)</h1>
      <p>
        On the left, enter valid timeblok program.
        On the right, the output(in ics format) will be displayed.
      </p>
      <div className="flex">
        <textarea value={leftText} onChange={(e) => setLeftText(e.target.value)} className="w-1/2" />
        <textarea value={rightText} onChange={(e) => setRightText(e.target.value)} className="w-1/2" />
      </div>
      <button className="mt-4" onClick={handleClick}>Button</button>
    </div>
 )

}

export default App
