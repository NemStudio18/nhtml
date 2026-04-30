package nhtml

import (
	"encoding/json"
	"io"
	"net/http"
)

const Version = "0.7.3-stable"

type Event struct {
	Handler     string                 `json:"handler"`
	SourceID    string                 `json:"source_id"`
	SessionID   string                 `json:"session_id"`
	Payload     string                 `json:"payload"`
	LastVersion uint32                 `json:"last_version"`
	Nodes       map[string]interface{} `json:"nodes"`
	Data        map[string]interface{} `json:"-"`
}

// Nhtml is the main entry point for the Go SDK
type Nhtml struct{}

func (n Nhtml) Patch() *Patch {
	return NewPatch()
}

func (n Nhtml) ParseEvent(r *http.Request) (*Event, error) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		return nil, err
	}

	var event Event
	if err := json.Unmarshal(body, &event); err != nil {
		return nil, err
	}

	if event.Payload != "" {
		var payloadData map[string]interface{}
		if err := json.Unmarshal([]byte(event.Payload), &payloadData); err == nil {
			event.Data = payloadData
		}
	}

	return &event, nil
}

func (n Nhtml) Send(w http.ResponseWriter, p *Patch) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(p)
}

func (n Nhtml) JoinRoom(w http.ResponseWriter, roomID string) {
	n.Send(w, NewPatch().JoinRoom(roomID))
}

func (n Nhtml) LeaveRoom(w http.ResponseWriter, roomID string) {
	n.Send(w, NewPatch().LeaveRoom(roomID))
}
