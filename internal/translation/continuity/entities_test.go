package continuity

import (
	"github.com/voocel/ainovel-cli/internal/translation/domain"
	"testing"
)

func TestAliasesKeepIdentityAcrossSpellingRevisionsAndDirectedPairs(t *testing.T) {
	rules := []domain.Rule{
		{ID: "a", Kind: "name", Status: "confirmed", Key: "スバル", Value: "Subaru", Evidence: []domain.Evidence{{Quote: "スバル"}}},
		{ID: "b", Kind: "name", Status: "confirmed", Key: "エミリア", Value: "Emilia", Evidence: []domain.Evidence{{Quote: "エミリア"}}},
		{ID: "secret", Kind: "name", Status: "candidate", Key: "秘密", Value: "Secret"},
	}
	entities := Entities(rules)
	if len(entities) != 2 {
		t.Fatal(entities)
	}
	a, b := CanonicalActor(entities, "Subaru"), CanonicalActor(entities, "エミリア")
	if a == b || a == "Subaru" {
		t.Fatal("aliases did not resolve distinct actors")
	}
	g := domain.Generation{Units: map[string]domain.Unit{
		"forward": accepted("forward", 1, 1, a, b, "tôi-cô"),
		"reverse": accepted("reverse", 1, 1, b, a, "tôi-cậu"),
	}}
	events := Context([]domain.Generation{g}, domain.Position{Volume: 2}, nil)
	if len(events) != 2 || events[0].Speaker == events[1].Speaker {
		t.Fatal("directed pair collapsed", events)
	}
	rules[0].Value = "Su-ba-ru"
	if CanonicalActor(Entities(rules), "Su-ba-ru") != a {
		t.Fatal("spelling revision changed identity")
	}
	if CanonicalActor(entities, "anonymous:chapter1:1") != "anonymous:chapter1:1" || CanonicalActor(entities, "Secret") != "Secret" {
		t.Fatal("unknown identity guessed")
	}
}

func TestAmbiguousAliasesDoNotMergeCharacters(t *testing.T) {
	entities := []domain.Entity{{ID: "a", Aliases: []string{"người đàn ông"}}, {ID: "b", Aliases: []string{"người đàn ông"}}}
	if CanonicalActor(entities, "người đàn ông") != "người đàn ông" {
		t.Fatal("ambiguous alias merged characters")
	}
	if CanonicalActor(entities, "a") != "a" {
		t.Fatal("explicit ID lost")
	}
}
