package epub

import (
	"bytes"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"strings"
)

type Node struct {
	Name     string
	NS       string
	Attr     []xml.Attr
	Children []*Node
	Text     string
	Locator  string
}

func local(name string) string {
	if i := strings.IndexByte(name, ':'); i >= 0 {
		return name[i+1:]
	}
	return name
}
func qname(n xml.Name) string {
	if n.Space != "" {
		return n.Space + ":" + n.Local
	}
	return n.Local
}
func (n *Node) Get(key string) string {
	for _, a := range n.Attr {
		if qname(a.Name) == key {
			return a.Value
		}
	}
	return ""
}
func (n *Node) GetLocal(key string) string {
	for _, a := range n.Attr {
		if a.Name.Local == key {
			return a.Value
		}
	}
	return ""
}
func (n *Node) Set(key, value string) {
	for i, a := range n.Attr {
		if qname(a.Name) == key {
			n.Attr[i].Value = value
			return
		}
	}
	parts := strings.SplitN(key, ":", 2)
	name := xml.Name{Local: key}
	if len(parts) == 2 {
		name = xml.Name{Space: parts[0], Local: parts[1]}
	}
	n.Attr = append(n.Attr, xml.Attr{Name: name, Value: value})
}
func (n *Node) Walk(fn func(*Node)) {
	fn(n)
	for _, c := range n.Children {
		c.Walk(fn)
	}
}
func (n *Node) Find(tag string) []*Node {
	var out []*Node
	n.Walk(func(c *Node) {
		if local(c.Name) == tag {
			out = append(out, c)
		}
	})
	return out
}
func (n *Node) AllText() string {
	if n.Name == "" {
		return n.Text
	}
	var b strings.Builder
	for _, c := range n.Children {
		b.WriteString(c.AllText())
	}
	return b.String()
}
func Parse(data []byte) (*Node, error) {
	d := xml.NewDecoder(bytes.NewReader(data))
	d.Strict = true
	d.Entity = map[string]string{"nbsp": "\u00a0"}
	root := &Node{Name: "#root", Locator: ""}
	stack := []*Node{root}
	namespaces := []map[string]string{{"xml": "http://www.w3.org/XML/1998/namespace"}}
	nodes := 0
	for {
		t, e := d.RawToken()
		if e == io.EOF {
			break
		}
		if e != nil {
			return nil, e
		}
		nodes++
		if nodes > 1000000 || len(stack) > 256 {
			return nil, errors.New("XML vượt giới hạn node/depth")
		}
		switch v := t.(type) {
		case xml.StartElement:
			ns := map[string]string{}
			for k, x := range namespaces[len(namespaces)-1] {
				ns[k] = x
			}
			for _, a := range v.Attr {
				if a.Name.Space == "xmlns" {
					ns[a.Name.Local] = a.Value
				} else if a.Name.Local == "xmlns" && a.Name.Space == "" {
					ns[""] = a.Value
				}
			}
			if v.Name.Space != "" && ns[v.Name.Space] == "" {
				return nil, fmt.Errorf("namespace chưa khai báo: %s", v.Name.Space)
			}
			for _, a := range v.Attr {
				if a.Name.Space != "" && a.Name.Space != "xmlns" && ns[a.Name.Space] == "" {
					return nil, fmt.Errorf("attribute namespace chưa khai báo: %s", a.Name.Space)
				}
			}
			p := stack[len(stack)-1]
			n := &Node{Name: qname(v.Name), NS: ns[v.Name.Space], Attr: v.Attr, Locator: fmt.Sprintf("%s/%d", p.Locator, len(p.Children))}
			p.Children = append(p.Children, n)
			stack = append(stack, n)
			namespaces = append(namespaces, ns)
		case xml.EndElement:
			if len(stack) < 2 || stack[len(stack)-1].Name != qname(v.Name) {
				return nil, errors.New("XML đóng tag không khớp")
			}
			stack = stack[:len(stack)-1]
			namespaces = namespaces[:len(namespaces)-1]
		case xml.CharData:
			p := stack[len(stack)-1]
			text := string(v)
			if len(stack) == 1 {
				if strings.TrimSpace(text) != "" {
					return nil, errors.New("text ngoài XML root")
				}
				continue
			}
			if len(p.Children) > 0 && p.Children[len(p.Children)-1].Name == "" {
				p.Children[len(p.Children)-1].Text += text
			} else {
				p.Children = append(p.Children, &Node{Text: text, Locator: fmt.Sprintf("%s/%d", p.Locator, len(p.Children))})
			}
		case xml.Directive:
			if strings.Contains(strings.ToUpper(string(v)), "ENTITY") {
				return nil, errors.New("XML entity declarations không được hỗ trợ")
			}
		}
	}
	if len(stack) != 1 {
		return nil, errors.New("XML chưa đóng")
	}
	elements := 0
	for _, n := range root.Children {
		if n.Name != "" {
			elements++
		}
	}
	if elements != 1 {
		return nil, errors.New("XML phải có một root element")
	}
	return root, nil
}
func Escape(s string) string {
	var b bytes.Buffer
	_ = xml.EscapeText(&b, []byte(s))
	return b.String()
}
func (n *Node) XML() string {
	if n.Name == "" {
		return Escape(n.Text)
	}
	var b strings.Builder
	if n.Name == "#root" {
		for _, c := range n.Children {
			b.WriteString(c.XML())
		}
		return b.String()
	}
	b.WriteString("<" + n.Name)
	for _, a := range n.Attr {
		b.WriteString(" " + qname(a.Name) + "=\"" + Escape(a.Value) + "\"")
	}
	b.WriteByte('>')
	for _, c := range n.Children {
		b.WriteString(c.XML())
	}
	b.WriteString("</" + n.Name + ">")
	return b.String()
}
func Serialize(root *Node) []byte {
	var b strings.Builder
	b.WriteString("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n")
	for _, n := range root.Children {
		b.WriteString(n.XML())
	}
	return []byte(b.String())
}
