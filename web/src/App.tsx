import { useState } from 'react'
import * as tb from "timeblok-js/dist/bundler"


function App() {
  const [leftText, setLeftText] = useState('2023-4-1\n9am do stuff')
  const [rightText, setRightText] = useState('Click compile to compile to ics')
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

  const export_ics = () => {
    let element = document.createElement('a');
    let file = new Blob([rightText], {type: 'text/plain'});
    element.href = URL.createObjectURL(file);
    element.download = "export.ics";
    document.body.appendChild(element);
    element.click();
  }

  return (
    <div className="container mx-auto text-center h-screen prose">
      <h1 className='h-1 block'>Timeblok Playground!</h1>
      <p>
        On the left, enter valid timeblok program.
        On the right, the output(in ics format) will be displayed.
      </p>
      <div className="flex">
        <div className='w-1/2'>
          TimeBlok Code <br/>
          <textarea className='w-full' value={leftText} onChange={(e) => setLeftText(e.target.value)}/>
        </div>
        <div className='w-1/2 ml-2'>
          ICS export <br/>
          <textarea className='w-full' value={rightText}/>
        </div>
      </div>
      <button className="btn btn-sm" onClick={handleClick}>Compile</button>
      <button className="btn btn-sm ml-2" onClick={export_ics}>Export to ICS</button>
    </div>
 )

}

export default App
