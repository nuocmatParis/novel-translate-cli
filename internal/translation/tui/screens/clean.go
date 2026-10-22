package screens

import "github.com/voocel/ainovel-cli/internal/translation/tui/theme"

func cleanDisplayText(value string) string {
	return theme.CleanText(value)
}
