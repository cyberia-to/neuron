use neuron_model::*;

fn controlled(id: u8, network: u8) -> Binding {
    Binding {
        subject: SubjectRef::Native([id; 32]),
        network: NetworkRef::Native([network; 32]),
        revision: 0,
        access: Access::Control,
        state: BindingState::Attached,
        policy: [8; 32],
        evidence: Some([9; 32]),
    }
}
fn context(b: &Binding) -> ActionContext {
    ActionContext {
        subject: b.subject.clone(),
        network: b.network.clone(),
        binding_revision: b.revision,
        prog: Some([1; 32]),
        invocation: Some([2; 32]),
        policy: b.policy,
        grant: [3; 32],
    }
}

#[test]
fn foreign_identity_is_domain_qualified_and_remains_observable_without_execution() {
    let a = SubjectRef::Foreign {
        domain: "cosmos:bostrom".into(),
        address: b"same-address".to_vec(),
    };
    let b = SubjectRef::Foreign {
        domain: "cosmos:space-pussy".into(),
        address: b"same-address".to_vec(),
    };
    assert_ne!(a, b);
    assert_eq!(a.native(), None);
    let mut registry = Attachments::default();
    let binding = Binding {
        subject: a,
        network: NetworkRef::Foreign {
            domain: "cosmos".into(),
            network: b"bostrom".to_vec(),
        },
        revision: 0,
        access: Access::Observe,
        state: BindingState::Attached,
        policy: [0; 32],
        evidence: None,
    };
    registry.apply(None, binding.clone()).unwrap();
    assert_eq!(
        registry.authorize_context(&context(&binding)),
        Err(IdentityError::Denied)
    );
    assert_eq!(registry.iter().count(), 1);
}

#[test]
fn admitted_context_keeps_subject_network_and_revision_after_selection_and_revoke() {
    let mut registry = Attachments::default();
    let first = controlled(1, 2);
    let second = controlled(3, 4);
    let admitted = context(&first);
    registry.apply(None, first.clone()).unwrap();
    registry.apply(None, second.clone()).unwrap();
    assert_eq!(
        registry.authorize_context(&admitted).unwrap().subject,
        first.subject
    );
    let mut wrong_network = admitted.clone();
    wrong_network.network = second.network.clone();
    assert_eq!(
        registry.authorize_context(&wrong_network),
        Err(IdentityError::Missing)
    );
    let mut revoked = first.clone();
    revoked.revision = 1;
    revoked.state = BindingState::Revoked;
    registry.apply(Some(0), revoked.clone()).unwrap();
    assert_eq!(
        registry.authorize_context(&admitted),
        Err(IdentityError::Denied)
    );
    assert_eq!(registry.apply(Some(0), first), Err(IdentityError::Conflict));
    revoked.revision = 2;
    revoked.state = BindingState::Attached;
    registry.apply(Some(1), revoked).unwrap();
    assert_eq!(
        registry.authorize_context(&admitted),
        Err(IdentityError::Denied)
    );
    assert!(registry.authorize_context(&context(&second)).is_ok());
}

#[test]
fn malformed_or_unproven_bindings_are_rejected_before_projection() {
    let mut registry = Attachments::default();
    let mut binding = controlled(1, 2);
    binding.evidence = None;
    assert_eq!(registry.apply(None, binding), Err(IdentityError::Denied));
    for domain in ["", "bad\ndomain", "spaces here"] {
        assert!(
            SubjectRef::Foreign {
                domain: domain.into(),
                address: vec![1]
            }
            .validate()
            .is_err()
        );
    }
    let oversized = SubjectRef::Foreign {
        domain: "cosmos".into(),
        address: vec![1; 257],
    };
    assert_eq!(oversized.validate(), Err(IdentityError::InvalidReference));
    let mut invalid = context(&controlled(1, 2));
    invalid.invocation = None;
    assert_eq!(invalid.validate(), Err(IdentityError::InvalidReference));
    assert!(Destination::View("landing".into()).validate().is_ok());
    assert_eq!(registry.iter().count(), 0);
}
