pub enum SmtpTemplate {
    ResidentInvite,
    NewAccount,
}

pub struct SmtpTemplateData {
    pub key: String,
    pub value: String,
}

pub fn smtp_get_template(template_type: SmtpTemplate, data: Vec<SmtpTemplateData>) -> String {
    let mut template_str = match template_type {
        SmtpTemplate::ResidentInvite => {
            include_str!("../../../res/mail/resident_invite.html").to_string()
        }
        SmtpTemplate::NewAccount => {
            include_str!("../../../res/mail/new_account_created.html").to_string()
        }
    };

    for parameter in data {
        template_str = template_str.replace(&parameter.key, &parameter.value);
    }

    template_str
}
