from chat import load_prompts
import json 

def store_prompt(fp):
    """Stores prompt in a json file"""
    dat = load_prompts()
    json.dump(dat, open(fp, 'w'))

if __name__ == "__main__":
    store_prompt("./prompt.json")