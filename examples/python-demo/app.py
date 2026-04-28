from fastapi import FastAPI, Request
from nhtml.patch import Patch
from nhtml.fastapi import nhtml_response, parse_event
import uvicorn
import sys
import os

# Ajout du chemin du SDK local pour le test
sys.path.append(os.path.abspath("../../sdk/python"))

app = FastAPI()

@app.post("/")
async def handle_nhtml(request: Request):
    data = await request.json()
    event = parse_event(data)
    
    patch = Patch.create()
    
    if event['handler'] == 'increment':
        # Simuler un compteur (normalement on utiliserait une DB ou session)
        current = int(event['value'] or 0)
        new_val = current + 1
        patch.set_text("counter_val", str(new_val))
        patch.set_style("counter_val", "color", "var(--primary)")
        
    elif event['handler'] == 'set_lang':
        lang = event['value']
        if lang == 'fr':
            patch.set_text("title", "Bienvenue sur NHTML (Python)")
        else:
            patch.set_text("title", "Welcome to NHTML (Python)")
            
    return await nhtml_response(patch)

if __name__ == "__main__":
    uvicorn.run(app, host="127.0.0.1", port=8000)
