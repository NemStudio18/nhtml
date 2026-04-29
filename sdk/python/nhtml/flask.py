from flask import jsonify
from .patch import Patch

def nhtml_response(patch: Patch):
    """
    Retourne une réponse compatible NHTML pour Flask.
    """
    return jsonify(patch.to_dict())

def parse_event(request_json: dict):
    """
    Extrait l'événement NHTML depuis les données JSON de la requête Flask.
    """
    payload = request_json.get('payload', '')
    data = {}
    try:
        import json
        data = json.loads(payload)
    except:
        pass

    return {
        'handler': request_json.get('handler'),
        'source_id': request_json.get('source_id'),
        'session_id': request_json.get('session_id'),
        'payload': payload,
        'data': data,
        'last_version': request_json.get('last_version'),
        'nodes': request_json.get('nodes', {})
    }
