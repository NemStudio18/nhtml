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
    return {
        'node_id': request_json.get('n-id'),
        'handler': request_json.get('n-handler'),
        'value': request_json.get('n-value'),
        'type': request_json.get('n-type'),
        'session_id': request_json.get('n-session-id')
    }
