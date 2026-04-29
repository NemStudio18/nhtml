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
    payload = request_data.get('payload', '')
    data = {}
    try:
        import json
        data = json.loads(payload)
    except:
        pass

    return {
        'handler': request_data.get('handler'),
        'source_id': request_data.get('source_id'),
        'session_id': request_data.get('session_id'),
        'payload': payload,
        'data': data,
        'last_version': request_data.get('last_version'),
        'nodes': request_data.get('nodes', {})
    }
