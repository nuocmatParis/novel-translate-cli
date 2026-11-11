#[derive(Debug, Clone)]
pub struct MockCharacter {
    pub name: String,
    pub original_name: String,
    pub alias: String,
    pub gender: String,
    pub role: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct MockTerminology {
    pub source_term: String,
    pub vietnamese_translation: String,
    pub notes: String,
    pub scope: String,
    pub source_file: String,
}

#[derive(Debug, Clone)]
pub struct MockAddressing {
    pub speaker: String,
    pub listener: String,
    pub self_reference: String,
    pub listener_reference: String,
    pub scope: String,
    pub context: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct MockRelationship {
    pub from_char: String,
    pub to_char: String,
    pub relation_type: String,
    pub notes: String,
}

#[derive(Debug, Clone)]
pub struct MockTimelineEvent {
    pub chapter: String,
    pub point: String,
    pub summary: String,
}
