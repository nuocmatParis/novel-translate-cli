package tui

import (
	"bytes"
	"fmt"
	"image"
	"image/color"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"
	"math"
	"os"
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/voocel/ainovel-cli/internal/translation/app"
)

type coverPreview struct {
	key     string
	text    string
	message string
	volume  app.VolumeCover
}

type coverLoadedMsg struct {
	token   uint64
	key     string
	volume  app.VolumeCover
	preview string
	message string
	err     error
}

type coverDetailsMsg struct {
	token    uint64
	root     string
	volume   app.VolumeCover
	columns  int
	rows     int
	useColor bool
	err      error
}

func loadCoverDetailsCmd(service *app.AppService, volumeID string, columns, rows int, token uint64) tea.Cmd {
	return func() tea.Msg {
		result, err := service.QueryVolumeCover(volumeID)
		root := ""
		if service.Config != nil {
			root = service.Config.Root
		}
		return coverDetailsMsg{token: token, root: root, volume: result, columns: columns, rows: rows, useColor: terminalSupportsColorPreview(), err: err}
	}
}

func renderCoverCmd(volume app.VolumeCover, columns, rows int, useColor bool, token uint64, key string) tea.Cmd {
	return func() tea.Msg {
		preview, err := renderCoverThumbnail(volume.Image, columns, rows, useColor)
		msg := coverLoadedMsg{token: token, key: key, volume: volume, err: err}
		if err != nil {
			msg.message = err.Error()
			return msg
		}
		msg.preview = preview
		return msg
	}
}

// renderCoverThumbnail fits the entire cover into terminal cells. Each cell
// represents two pixels vertically, so the cover keeps its aspect ratio.
func renderCoverThumbnail(data []byte, columns, rows int, useColor bool) (string, error) {
	if columns < 1 || rows < 1 {
		return "", fmt.Errorf("kích thước thumbnail không hợp lệ")
	}
	config, _, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return "", fmt.Errorf("không đọc được ảnh bìa: %w", err)
	}
	if config.Width < 1 || config.Height < 1 || config.Width > 10000 || config.Height > 10000 || int64(config.Width)*int64(config.Height) > 16_000_000 {
		return "", fmt.Errorf("kích thước ảnh bìa vượt giới hạn xem trước")
	}
	img, _, err := image.Decode(bytes.NewReader(data))
	if err != nil {
		return "", fmt.Errorf("không giải mã được ảnh bìa: %w", err)
	}
	bounds := img.Bounds()
	pixelWidth, pixelHeight := columns, rows*2
	scale := math.Min(float64(pixelWidth)/float64(bounds.Dx()), float64(pixelHeight)/float64(bounds.Dy()))
	fitW := max(1, int(math.Round(float64(bounds.Dx())*scale)))
	fitH := max(1, int(math.Round(float64(bounds.Dy())*scale)))
	left, top := (pixelWidth-fitW)/2, (pixelHeight-fitH)/2

	background := color.RGBA{R: 11, G: 27, B: 38, A: 255}
	pixels := make([]color.RGBA, pixelWidth*pixelHeight)
	for i := range pixels {
		pixels[i] = background
	}
	for y := 0; y < fitH; y++ {
		sourceY := bounds.Min.Y + min(bounds.Dy()-1, y*bounds.Dy()/fitH)
		for x := 0; x < fitW; x++ {
			sourceX := bounds.Min.X + min(bounds.Dx()-1, x*bounds.Dx()/fitW)
			c := color.NRGBAModel.Convert(img.At(sourceX, sourceY)).(color.NRGBA)
			a := uint32(c.A)
			inverse := uint32(255) - a
			pixels[(top+y)*pixelWidth+left+x] = color.RGBA{
				R: uint8((uint32(c.R)*a + uint32(background.R)*inverse) / 255),
				G: uint8((uint32(c.G)*a + uint32(background.G)*inverse) / 255),
				B: uint8((uint32(c.B)*a + uint32(background.B)*inverse) / 255),
				A: 255,
			}
		}
	}

	var out strings.Builder
	for y := 0; y < pixelHeight; y += 2 {
		for x := 0; x < pixelWidth; x++ {
			upper := pixels[y*pixelWidth+x]
			lower := pixels[(y+1)*pixelWidth+x]
			if useColor {
				fmt.Fprintf(&out, "\x1b[38;2;%d;%d;%dm\x1b[48;2;%d;%d;%dm▀\x1b[0m", upper.R, upper.G, upper.B, lower.R, lower.G, lower.B)
			} else {
				lum := (299*int(upper.R) + 587*int(upper.G) + 114*int(upper.B) + 299*int(lower.R) + 587*int(lower.G) + 114*int(lower.B)) / 2000
				shade := "@%#*+=-:. "
				out.WriteByte(shade[min(len(shade)-1, lum*(len(shade)-1)/255)])
			}
		}
		if y+2 < pixelHeight {
			out.WriteByte('\n')
		}
	}
	return out.String(), nil
}

func terminalSupportsColorPreview() bool {
	return os.Getenv("NO_COLOR") == "" && !strings.EqualFold(os.Getenv("TERM"), "dumb")
}
