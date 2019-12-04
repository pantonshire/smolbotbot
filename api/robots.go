package api

// Robot represents a small robot stored in the database.
type Robot struct {
	ID     int `json:"id"`
	Number int `json:"number"`

	Name struct {
		Full   string `json:"full"`
		Prefix string `json:"prefix"`
	} `json:"name"`

	Tweet struct {
		ID        string `json:"id"`
		Timestamp int    `json:"timestamp"`
	} `json:"tweet"`

	Image struct {
		URL string `json:"url"`
		Alt string `json:"alt"`
	} `json:"image"`

	Description string `json:"description"`

	Tags string `json:"tags"`
}

// Link returns the URL of the original tweet that robot originates from.
func (robot *Robot) Link() string {
	return "https://twitter.com/smolrobots/status/" + robot.Tweet.ID
}
