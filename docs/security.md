<a id="security-and-privacy"></a>

# Security and access boundaries

This SDK reads supported public files through the local PC/SC service. A
successful read provides decoded data; it does not authenticate the card or
verify the identity of the person presenting it.

<a id="sdk-guarantees"></a>

## Local data handling

- The library makes no network requests and writes no cardholder data to disk.
- Error messages do not include decoded cardholder field values.
- The diagnostic CLI redacts reads by default. Its explicit
  `--show-personal-data` option displays basic identity values locally.
- Snapshots remain in application memory until released. Dropping them does
  not guarantee memory zeroization or removal of copies held by the application.

## Fields not read by this SDK

The SDK does not request these address and contact fields:

- Home address details
- Work address details
- Resident phone number
- Mobile phone number
- Email address

The imported implementation's access checks encountered protected files. This
SDK has no authentication or secure-messaging support and leaves these fields
outside its data model. Their absence does not mean the card has no such data.

The exported `PROTECTED_AND_SKIPPED_FIELDS` constant lists these fields. They
have no getter, snapshot field, or per-group status. See the generated
[field reference](field-reference.md#fields-not-read-by-this-sdk).

Fingerprint templates and family-book records are also not read. Some supported
public groups, including photographs, can themselves be protected on an
individual card. Those attempted reads receive a group status as described in
[errors and read statuses](error-handling.md).

<a id="caller-responsibilities"></a>

## Application responsibilities

- Request only the groups your operation needs.
- Avoid logging snapshots or attaching them to crash reports. Their `Debug`
  and `Serialize` implementations include personal data.
- Clear displayed data and retained copies when they are no longer needed,
  including after card removal in an interactive reader application.
- Define access controls and retention rules for any data you choose to store
  or transmit.

## Authenticity boundary

An ATR match identifies a documented chip family for compatibility purposes.
It does not prove a card is genuine. The
[EIDA C++ developer guide](sources.md#c-developer-guide), section 5.7, explicitly
distinguishes ATR checks from the toolkit's separate genuineness operation.

This SDK does not implement that operation, verify digital signatures, or
establish a certificate trust policy. Applications needing verified identity
must address those requirements through an appropriate verification integration.
Reading a photograph or holder-signature image is not signature verification.
