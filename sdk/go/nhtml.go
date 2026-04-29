package nhtml

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
)

const Version = "0.7.1"

type Event struct {
	Handler   string                 `json:"handler"`
	Payload   string                 `json:"payload"`
	Transport string                 `json:"transport"`
	Data      map[string]interface{} `json:"-"`
}

func ParseEvent(r *http.Request) (*Event, error) {
	body, err := io.ReadAll(r.Body)
	if err != nil {
		return nil, err
	}

	var event Event
	if err := json.Unmarshal(body, &event); err != nil {
		return nil, err
	}

	// Parse internal payload which is usually a JSON string
	if event.Payload != "" {
		var payloadData map[string]interface{}
		if err := json.Unmarshal([]byte(event.Payload), &payloadData); err == nil {
			event.Data = payloadData
		}
	}

	return &event, nil
}

func SendResponse(w http.ResponseWriter, p *Patch) {
	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(p)
}

// Gin Handler helper
func GinHandler(c interface{}, handler func(*Event) *Patch) {
	// This is a placeholder since we don't want to force gin dependency here
	// but developers can easily adapt it.
	fmt.Println("NHTML Go SDK initialized")
}
