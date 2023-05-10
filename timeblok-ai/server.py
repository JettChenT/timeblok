from fastapi import FastAPI, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from chat import load_prompts, complete, parse_results, set_openai_api
from pydantic import BaseModel, BaseSettings
from typing import List, Dict, Tuple


app = FastAPI()
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

class Settings(BaseSettings):
    openai_api_key: str
    openai_api_base: str|None = None
    access_key: str

    class Config:
        env_file = ".env"

class AuthInfo(BaseModel):
    access_key: str|None = None
    openai_key: str|None = None
    
    def get_key(self):
        if self.access_key == settings.access_key:
            return settings.openai_api_key
        return self.openai_key


class ChatHistoryEntry(BaseModel):
    role: str
    content: str

class ChatHistory(BaseModel):
    entries: List[ChatHistoryEntry]
    auth: AuthInfo

settings = Settings()

@app.get("/")
async def root():
    return {"message": "Hello World"}



@app.post("/chat")
async def chat(chat_history: ChatHistory):
    key = chat_history.auth.get_key()
    if key is None:
        raise HTTPException(status_code=400, detail="Access key or OpenAI API key must be provided.")
    set_openai_api(key, settings.openai_api_base)
    prompts = load_prompts()
    prompts.extend([{'role':c.role, "content":c.content} for c in chat_history.entries])
    completion, content = complete(prompts)
    parsed = parse_results(content)
    return {
        "content": content,
        "parsed": parsed,
        "completion": completion.choices[0].message
    }
