import openai
import dotenv
import os
import datetime
import re
import tempfile
import subprocess
from rich.console import Console
from rich.markdown import Markdown
from rich import print

def set_openai_api_env():
    dotenv.load_dotenv()
    if os.getenv("OPENAI_API_BASE") is not None:
        openai.api_base = os.getenv("OPENAI_API_BASE")
    openai.api_key = os.getenv("OPENAI_API_KEY")

def set_openai_api(key, base=None):
    openai.api_key = key
    if base is not None:
        openai.api_base = base

def load_examples():
    # read all files in examples/folder
    for filename in os.listdir('./examples'):
        lines = open(os.path.join('./examples', filename)).readlines()
        prompt = tb =  ""
        flag = 0
        for l in lines:
            if l == '---\n':
                flag=0
                yield (prompt, tb)
                prompt = tb = ""
            elif l == '+++\n':
                flag=1
            elif flag: tb+=l
            else: prompt+=l

def load_prompts():
    prompt = open('./prompt.md').read()
    today = datetime.date.today().strftime("%Y-%m-%d")
    prompt += f"\nToday's date is {today}"
    messages=[{"role": "system", "content": prompt}]
    for example in load_examples():
        messages.append({"role": "user", "content": example[0]})
        messages.append({"role": "assistant", "content": example[1]})
    return messages

def parse_results(markdown):
    code_blocks = re.findall("```(timeblok)\n(.*?)```", markdown, flags=re.DOTALL)
    if code_blocks:
        return code_blocks[0][1]
    return None

def open_in_calendar(timeblok):
    with tempfile.NamedTemporaryFile(suffix='.tb') as fp:
        # Write timeblok content to the temporary file
        fp.write(timeblok.encode())
        fp.flush()
        # Run the system command to open the timeblok file in a calendar
        try:
            subprocess.run(['timeblok', fp.name, '-o'], check=True)
        except subprocess.CalledProcessError:
            print("Error: failed to open timeblok in calendar")

def complete(messages):
    completion = openai.ChatCompletion.create(
            model="gpt-3.5-turbo",
            messages=messages,
        )
    content = completion.choices[0].message.content
    return completion, content
# create a basic input() based chatbot
def repl():
    set_openai_api_env()
    messages = load_prompts()
    console = Console()
    last_timeblok = None
    # Add help message in the beginning
    print("Welcome to Timeblok! \n Use '/n' to create a new timeblok, \n '/e' to edit the returned timeblok, \n '/s' to open the last returned timeblok, \n or '/q' to quit\n")
    while True:
        # prompt the user with the option to use "/n", "/e", "/s", or "/q"
        user_input = input("Enter command (/n, /e, /s, /q): \n")
        
        if user_input == "/q":
            # quit the program if user input is /q
            break
        elif user_input.startswith("/n"):
            # replace the first word with "new:"
            user_input = "new:" + user_input[2:]
        elif user_input.startswith("/e"):
            # replace the first word with "edit:"
            user_input = "edit:" + user_input[2:]
        elif user_input == "/s":
            # if user selects /s and there was a valid previous timeblok
            if last_timeblok is not None:
                open_in_calendar(last_timeblok)
                continue
        else:
            user_input = 'new:' + user_input
        
        messages.append({"role": "user", "content": user_input})
        completion, content = complete(messages)
        res = parse_results(content)
        if res is not None:
            last_timeblok = res
        markdown = Markdown(content)
        console.print(markdown)
        messages.append(completion.choices[0].message)

if __name__ == "__main__":
    repl()