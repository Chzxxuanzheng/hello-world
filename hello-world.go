package main

import (
	"fmt"
	"math/rand"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"
	"unicode/utf8"
)

func getLangCode() string {
	lang := os.Getenv("LANG")
	if lang == "" {
		lang = "en_US"
	}
	lang = strings.Split(lang, ".")[0]
	return strings.ToLower(strings.ReplaceAll(lang, "_", "-"))
}

func readTexts(langCode string) string {
	path := filepath.Join("i18n", langCode+".txt")
	content, err := os.ReadFile(path)
	if err != nil {
		content, _ = os.ReadFile(filepath.Join("i18n", "en-us.txt"))
	}
	return string(content)
}

func getFiles(codeDir string) []string {
	files := []string{}
	entries, _ := os.ReadDir(codeDir)
	for _, entry := range entries {
		if !entry.IsDir() {
			files = append(files, entry.Name())
		}
	}
	return files
}

func getTerminalWidth() int {
	cmd := exec.Command("tput", "cols")
	out, err := cmd.Output()
	if err != nil {
		return 80
	}
	w := strings.TrimSpace(string(out))
	var width int
	fmt.Sscanf(w, "%d", &width)
	if width <= 0 {
		return 80
	}
	return width
}

func renderFrame(texts string, preRun func(), i int) bool {
	notEnd := false
	start := time.Now()
	columns := getTerminalWidth()
	fmt.Print("\033[H")
	preRun()
	lines := strings.Split(texts, "\n")
	for l, text := range lines {
		textList := []struct {
			w  int
			ch string
		}{}
		for _, ch := range text {
			if ch == '\t' {
				textList = append(textList, struct {
					w  int
					ch string
				}{8, string(ch)})
				for k := 0; k < 7; k++ {
					textList = append(textList, struct {
						w  int
						ch string
					}{0, ""})
				}
			} else if utf8.RuneLen(ch) == 3 {
				textList = append(textList, struct {
					w  int
					ch string
				}{2, string(ch)})
				textList = append(textList, struct {
					w  int
					ch string
				}{0, ""})
			} else {
				textList = append(textList, struct {
					w  int
					ch string
				}{1, string(ch)})
			}
		}
		chars := make([]string, columns)
		for idx := range chars {
			chars[idx] = " "
		}
		for j, item := range textList {
			if item.ch == "" {
				continue
			}
			t := float64(i)/100 - float64(j)*0.05 - float64(l)*0.5
			if t > 1 {
				if j+item.w-1 < columns {
					chars[j] = item.ch
					for k := 1; k < item.w; k++ {
						chars[j+k] = ""
					}
				}
				continue
			}
			notEnd = true
			if t < 0 {
				continue
			}
			n := int((1 - t) * (1 - t) * (1 - t) * float64(columns))
			if j+n+item.w-1 < columns {
				chars[j+n] = item.ch
				for k := 1; k < item.w; k++ {
					chars[j+n+k] = ""
				}
			}
		}
		fmt.Println(strings.Join(chars, ""))
	}
	os.Stdout.Sync()
	elapsed := time.Since(start)
	if elapsed < 10*time.Millisecond {
		time.Sleep(10*time.Millisecond - elapsed)
	}
	return notEnd
}

func main() {
	langCode := getLangCode()
	texts := readTexts(langCode)
	files := getFiles("code")
	fmt.Print("\033[?1049h\033[?25l")
	defer fmt.Print("\033[?25h\033[?1049l")
	defer fmt.Println(texts)
	var i int
	for i = 0; renderFrame(texts, func() {}, i); i++ {
	}
	for {
		randomFile := files[rand.Intn(len(files))]
		code, _ := os.ReadFile(filepath.Join("code", randomFile))
		codeStr := strings.ReplaceAll(string(code), "$$$", strings.Split(texts, "\n")[0])
		fmt.Print("\033[H\033[2J")
		i = 0
		for renderFrame(codeStr, func() { fmt.Println(texts) }, i) {
			i++
		}
		time.Sleep(1 * time.Second)
	}
}
