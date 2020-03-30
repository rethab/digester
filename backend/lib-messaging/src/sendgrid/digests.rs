use super::super::Env;
use super::*;

pub fn create_subject_for_list(env: &Env, list_name: &str) -> String {
    let mut subject = String::new();

    if *env != Env::Prod {
        subject.push_str(&format!("[{:?}] ", env));
    }

    subject.push_str("Digests from ");
    subject.push_str(list_name);
    subject
}

pub fn create_subject(env: &Env, subs: &[SendgridSubscription]) -> String {
    let mut subject = String::new();

    if *env != Env::Prod {
        subject.push_str(&format!("[{:?}] ", env));
    }

    subject.push_str("Digests from ");

    let max_len = 50; // mailjet says too long subjects could be suspicious/spammy
    let mut there_would_be_more = false;
    let mut added_one = false;

    for sub in subs {
        if subject.len() + sub.title.len() > max_len {
            there_would_be_more = true;
        } else {
            if added_one {
                subject.push_str(", ")
            }
            subject.push_str(&sub.title);
            added_one = true;
        }
    }

    if !added_one {
        if let Some(sub) = subs.iter().next() {
            subject.push_str(&sub.title);
        }
    } else if there_would_be_more {
        subject.push_str(" and more");
    }

    subject
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limit_subject_length_by_not_adding_very_long() {
        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let sub2 = SendgridSubscription::new("golang/tools", Vec::new());
        let sub3 = SendgridSubscription::new(
            "ohmylongorganisationname/ohmylongrepositoryname",
            Vec::new(),
        );
        let sub4 = SendgridSubscription::new("node/node", Vec::new());

        let actual = create_subject(&Env::Prod, &[sub1, sub2, sub3, sub4]);
        let expected = "Digests from kubernetes/kubernetes, golang/tools and more".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn show_long_subject_if_only_one() {
        let sub1 = SendgridSubscription::new(
            "ohmyverylongorganisationname/ohmyverylongrepositoryname",
            Vec::new(),
        );

        let actual = create_subject(&Env::Prod, &[sub1]);
        let expected =
            "Digests from ohmyverylongorganisationname/ohmyverylongrepositoryname".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn dont_show_and_more() {
        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let sub2 = SendgridSubscription::new("golang/tools", Vec::new());
        let actual = create_subject(&Env::Prod, &[sub1, sub2]);
        let expected = "Digests from kubernetes/kubernetes, golang/tools".to_owned();
        assert_eq!(expected, actual)
    }

    #[test]
    fn prepend_env_to_subject_in_dev_and_stg() {
        // dev
        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let actual = create_subject(&Env::Dev, &[sub1]);
        let expected = "[Dev] Digests from kubernetes/kubernetes".to_owned();
        assert_eq!(expected, actual);

        let sub1 = SendgridSubscription::new("kubernetes/kubernetes", Vec::new());
        let actual = create_subject(&Env::Stg, &[sub1]);
        let expected = "[Stg] Digests from kubernetes/kubernetes".to_owned();
        assert_eq!(expected, actual)
    }
}
