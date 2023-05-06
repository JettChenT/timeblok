# LLM based client for timeblok

## Usage

Step 0: Make sure you have `timeblok` executable installed on your system. Use `cargo install timeblok-cli` to install if not.

Step 1: install requirements

 `pip install -r requirements.txt`

Step 2: create `.env` file and insert environs:

```env
OPENAI_API_KEY={YOUR_OPENAI_KEY}
OPENAI_API_BASE={optional, custom openai endpoint}
```

Step 3: run the program

`python chat.py`
