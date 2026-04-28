from fastapi import Request
from fastapi.responses import JSONResponse
from .patch import Patch

async def nhtml_response(patch: Patch):
    """
    Retourne une réponse compatible NHTML pour FastAPI.
    """
    return JSONResponse(content=patch.to_dict())

def parse_event(request_data: dict):
    """
    Extrait l'événement NHTML depuis les données JSON de la requête.
    """
    return {
        'node_id': request_data.get('n-id'),
        'handler': request_data.get('n-handler'),
        'value': request_data.get('n-value'),
        'type': request_data.get('n-type'),
        'session_id': request_data.get('n-session-id')
    }
