package utils

import (
	"database/sql/driver"
	"time"
)

const DateTimeFormat = "2006-01-02 15:04:05"
const DateFormat = "2006-01-02"

type DateTime struct {
	time.Time
}

func Now() DateTime {
	return DateTime{Time: time.Now()}
}

// today 00:00:00
func Today() time.Time {
	return NormalizeDate(time.Now())
}

// normalize date to 00:00:00
func NormalizeDate(t time.Time) time.Time {
	return time.Date(t.Year(), t.Month(), t.Day(), 0, 0, 0, 0, time.Local)
}

// parse date to 00:00:00
func ParseDate(date string) (time.Time, error) {
	t, err := time.ParseInLocation(DateFormat, date, time.Local)
	if err != nil {
		return time.Time{}, err
	}

	return NormalizeDate(t), nil
}

// ent GoType method
func (d DateTime) Value() (driver.Value, error) {
	return d.Time, nil
}

func (d *DateTime) Scan(value interface{}) error {
	if value == nil {
		d.Time = time.Time{}
		return nil
	}

	switch v := value.(type) {
	case time.Time:
		d.Time = v
		return nil
	case string:
		t, err := time.Parse(DateTimeFormat, v)
		if err != nil {
			return err
		}

		d.Time = t
		return nil
	case []byte:
		t, err := time.Parse(DateTimeFormat, string(v))
		if err != nil {
			return err
		}

		d.Time = t
		return nil
	}

	return nil
}

func (d DateTime) MarshalJSON() ([]byte, error) {
	return []byte(`"` + d.Format(DateTimeFormat) + `"`), nil
}

func (d *DateTime) UnmarshalJSON(data []byte) error {
	if string(data) == "null" || string(data) == `""` {
		return nil
	}

	t, err := time.ParseInLocation(
		`"`+DateTimeFormat+`"`,
		string(data),
		time.Local,
	)

	if err != nil {
		return err
	}

	d.Time = t
	return nil
}
