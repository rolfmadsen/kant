use crate::features::model::ui::state::{
    MetadataField, ModelMetadataModalState, META_BTN_CANCEL, META_BTN_SAVE,
};
use crate::features::model::{ApprovalStatus, ModelScope, ModelStatus};
use crate::ui::app::Message;
use crate::ui::focus::FocusCycle;
use crate::ui::theme::{
    focused_button_style, modal_backdrop_style, modal_card_style, modern_input_style,
    primary_button_style, secondary_button_style, ThemeColors,
};
use iced::widget::{
    button, column, container, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

pub fn view_metadata_modal<'a>(
    meta_state: &'a ModelMetadataModalState,
    modal_focus: &FocusCycle,
) -> Element<'a, Message> {
    let title_row = row![
        text("📋 Modelomslag & Metadata (FDA Tabel D & E)")
            .size(17)
            .color(ThemeColors::SLATE_900),
        Space::new().width(Length::Fill),
        button(text("✕").size(13))
            .style(secondary_button_style)
            .on_press(Message::CloseMetadataModal)
            .padding([3, 7]),
    ]
    .align_y(Alignment::Center);

    let subtitle = text("Rediger overordnede metadata for modelprojektet jf. FDA Modelreglerne:")
        .size(12)
        .color(ThemeColors::TEXT_MUTED);

    // 1. Modelnavn
    let name_field = column![
        text("Modelnavn *").size(12).color(ThemeColors::SLATE_700),
        text_input("Nyt FDA Modelprojekt", &meta_state.name)
            .id("meta_name")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::Name, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4);

    // 2. Beskrivelse
    let desc_field = column![
        text("Modelbeskrivelse")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "Formål og omfang jf. FDA Modelreglerne...",
            &meta_state.description
        )
        .id("meta_desc")
        .style(modern_input_style)
        .on_input(|val| Message::UpdateMetadataField(MetadataField::Description, val))
        .padding(8)
        .width(Length::Fill),
    ]
    .spacing(4);

    // 3. Model-URI
    let uri_field = column![
        text("Model-URI *").size(12).color(ThemeColors::SLATE_700),
        text_input("https://data.gov.dk/model/core/...", &meta_state.uri)
            .id("meta_uri")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::Uri, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4);

    // 4. Livscyklus & Godkendelse (Tabel D & E)
    let status_pick = pick_list(
        &ModelStatus::ALL[..],
        Some(meta_state.status),
        Message::UpdateMetadataStatus,
    )
    .padding(7)
    .width(Length::Fill);

    let status_field = column![
        text("Modelstatus (Tabel E)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        status_pick,
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let approval_pick = pick_list(
        &ApprovalStatus::ALL[..],
        Some(meta_state.approval_status),
        Message::UpdateMetadataApprovalStatus,
    )
    .padding(7)
    .width(Length::Fill);

    let approval_field = column![
        text("Godkendelsesstatus")
            .size(12)
            .color(ThemeColors::SLATE_700),
        approval_pick,
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let status_approval_row = row![status_field, approval_field].spacing(12);

    let scope_pick = pick_list(
        &ModelScope::ALL[..],
        Some(meta_state.model_scope),
        Message::UpdateMetadataScope,
    )
    .padding(7)
    .width(Length::Fill);

    let scope_field = column![
        text("Modelomfang").size(12).color(ThemeColors::SLATE_700),
        scope_pick,
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let approved_by_field = column![
        text("Godkendt af").size(12).color(ThemeColors::SLATE_700),
        text_input("f.eks. FDA Fagråd eller Styrelse", &meta_state.approved_by)
            .id("meta_approved_by")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::ApprovedBy, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let scope_approved_row = row![scope_field, approved_by_field].spacing(12);

    // 5. Ansvar & Versionering
    let domain_field = column![
        text("Emneområde (§26)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input("f.eks. Vej og Trafik", &meta_state.domain_area)
            .id("meta_domain_area")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::DomainArea, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let org_field = column![
        text("Ansvarlig organisation")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "f.eks. Styrelsen for Dataforsyning eller Vejdirektoratet",
            &meta_state.responsible_org,
        )
        .id("meta_responsible_org")
        .style(modern_input_style)
        .on_input(|val| Message::UpdateMetadataField(MetadataField::ResponsibleOrg, val))
        .padding(8)
        .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let domain_org_row = row![domain_field, org_field].spacing(12);

    let version_field = column![
        text("Version").size(12).color(ThemeColors::SLATE_700),
        text_input("0.1.0", &meta_state.version)
            .id("meta_version")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::Version, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let lang_field = column![
        text("Sprog").size(12).color(ThemeColors::SLATE_700),
        text_input("da", &meta_state.language)
            .id("meta_language")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::Language, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let date_field = column![
        text("Senest ændret (ISO)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input("YYYY-MM-DD", &meta_state.date_modified)
            .id("meta_date_modified")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::DateModified, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let version_lang_row = row![version_field, lang_field, date_field].spacing(12);

    let notes_field = column![
        text("Revisionsnote").size(12).color(ThemeColors::SLATE_700),
        text_input(
            "f.eks. Væsentlige ændringer i relationer efter offentlig høring",
            &meta_state.version_notes,
        )
        .id("meta_version_notes")
        .style(modern_input_style)
        .on_input(|val| Message::UpdateMetadataField(MetadataField::VersionNotes, val))
        .padding(8)
        .width(Length::Fill),
    ]
    .spacing(4);

    // 6. Proveniens & Lovgrundlag
    let legal_field = column![
        text("Lovgrundlag (ELI URI'er, kommasepareret)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "https://www.retsinformation.dk/eli/lta/...",
            &meta_state.legal_sources,
        )
        .id("meta_legal_sources")
        .style(modern_input_style)
        .on_input(|val| Message::UpdateMetadataField(MetadataField::LegalSources, val))
        .padding(8)
        .width(Length::Fill),
    ]
    .spacing(4);

    let source_field = column![
        text("Kilde (ekstern standard)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input("f.eks. ISO 19107 eller INSPIRE", &meta_state.source)
            .id("meta_source")
            .style(modern_input_style)
            .on_input(|val| Message::UpdateMetadataField(MetadataField::Source, val))
            .padding(8)
            .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let derived_field = column![
        text("Udledt af (URI)")
            .size(12)
            .color(ThemeColors::SLATE_700),
        text_input(
            "https://data.gov.dk/model/core/...",
            &meta_state.was_derived_from,
        )
        .id("meta_derived_from")
        .style(modern_input_style)
        .on_input(|val| Message::UpdateMetadataField(MetadataField::WasDerivedFrom, val))
        .padding(8)
        .width(Length::Fill),
    ]
    .spacing(4)
    .width(Length::FillPortion(1));

    let source_derived_row = row![source_field, derived_field].spacing(12);

    let is_cancel_focused = modal_focus.is_focused(META_BTN_CANCEL);
    let is_save_focused = modal_focus.is_focused(META_BTN_SAVE);

    // 7. Action knapper
    let actions = row![
        Space::new().width(Length::Fill),
        button(text("Annuller").size(12))
            .style(focused_button_style(
                secondary_button_style,
                is_cancel_focused,
            ))
            .on_press(Message::CloseMetadataModal)
            .padding([6, 14]),
        button(text("💾 Gem Omslag").size(12))
            .style(focused_button_style(primary_button_style, is_save_focused))
            .on_press(Message::SaveMetadataModal)
            .padding([6, 16]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let form_col = column![
        name_field,
        desc_field,
        uri_field,
        status_approval_row,
        scope_approved_row,
        domain_org_row,
        version_lang_row,
        notes_field,
        legal_field,
        source_derived_row,
    ]
    .spacing(12);

    let scrollable_form = scrollable(form_col).height(Length::Fixed(460.0));

    let dialog_col = column![
        title_row,
        subtitle,
        Space::new().height(4),
        scrollable_form,
        Space::new().height(4),
        actions,
    ]
    .spacing(10);

    let modal_card = container(dialog_col)
        .style(modal_card_style)
        .padding(24)
        .width(Length::Fixed(640.0));

    container(modal_card)
        .style(modal_backdrop_style)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
