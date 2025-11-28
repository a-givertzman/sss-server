# sss-server | API Reference

## Description to Event structure

Event consits of three parts:
- Start field SYN = 22 (0x16)
- Header fields
- Data

**Bytes order**: All fields - Endian.little

**Event Header**
- Id        (4 bytes)
- Content   (1 byte)
- Cot       (1 byte)
- QueryId   (4 bytes)
- Size      (4 bytes)

**Event data** of size specified in the Size field in bytes
- Payload   (Size)

### Field **Id**

Size: 4 bytes

Field used to identify the Event. This number is defined in the request event, and will be the same in all returned events.

**Example:**

| Client                                |  Direction |          Server
|                 :---:                 |   :---:    |           :---:
| Event(id: 111, Cot::Act, Query Calc)  |    -->     |  
|                                       |    <--     |  Event(id: 111, Cot::Inf, Reply Calc:Ongoing)
|                                       |    <--     |  Event(id: 111, Cot::Inf, Reply Calc:Done)

### Field **Content**

**Size: 1 byte**

Field used to specify the kind of content stored in current Event.
Sent request and returned response shouldn't have same Content.

**Avaliable variants**

- Any       = 00 (0x00)   - Not implemented
- Bool      = 08 (0x08)   - Not implemented
- Bytes     = 02 (0x02)   - Data field contains raw bytes
- Duration  = 49 (0x31)   - Not implemented
- Empty     = 01 (0x01)   - Data field hasn't data
- F32       = 32 (0x20)   - Not implemented
- F64       = 33 (0x21)   - Not implemented
- I16       = 24 (0x18)   - Not implemented
- I32       = 25 (0x19)   - Not implemented
- I64       = 26 (0x1A)   - Not implemented
- Json      = 38 (0x26)   - Data field contains JSON string
- String    = 40 (0x28)   - Not implemented
- Timestamp = 48 (0x30)   - Not implemented
- U16       = 16 (0x10)   - Not implemented
- U32       = 17 (0x11)   - Not implemented
- U64       = 18 (0x12)   - Not implemented

### Field **Cot**

**Size: 1 byte**

Field used to specify the Cause and direction of transmission.
Allows send request or send command, return response or error, or just send and information Event.

**Avaliable variants**

- Inf      = 0b_0000_0010 = 2   (0x2),
- Act      = 0b_0000_0100 = 4   (0x4),
- ActCon   = 0b_0000_1000 = 8   (0x8),
- ActErr   = 0b_0001_0000 = 16  (0x10),
- Req      = 0b_0010_0000 = 32  (0x20),
- ReqCon   = 0b_0100_0000 = 64  (0x40),
- ReqErr   = 0b_1000_0000 = 128 (0x80),

**Inf** - Any information sent from Server to any Client, such event can be sent spontaneouselly without request from the Client, also can be sent as additional information while the request performing
**Act** - Command sent from Client to the Server. Optionally can be confirmed by the Event with `ActCon` or `ActErr`, but response on such Event is not required
**ActCon** - Optional positive confirmation on `Act`
**ActErr** - Optional confirmation in case of error ocured on `Act`
**Req** - Request sent from Client to the Server (or wise wersa). Response requared.
    - Confirmed by ReqCon - if response contains some positive result
    - Confirmed by ReqErr - if response contains error info
**ReqCon** - Positive confirmation on `Req`
**ReqErr** - Confirmation in case of error ocured on `Req`

### Field **QueryId**

**Size: 4 bytes**

Field used to specify exact query name

Bellow are listed Event queries supported by the SSS-Server V x.y.z

## Events

### Common Events

- **Empty**

- **Calculus**

    Performs complete calculations algorithm
    - **Request**
        - Content: Json
        - Cot: Act
        - QueryId: Calculus
        - data: {ship_id: int, project_id: String}
    - Response 'Ongoing' (optional)
        - Content: Json
        - Cot: Inf
        - QueryId: Calculus
        - data: {status: CalculusStatus::Ongoing}
    - Response 'Canceled' (optional)
        - Content: Json
        - Cot: Inf
        - QueryId: Calculus
        - data: {status: CalculusStatus::Canceled}
    - **Response 'Done'**
        - Content: Json
        - Cot: ActCon
        - QueryId: Calculus
        - data: {status: CalculusStatus::Done}
    - Error
        - Content: Json
        - Cot: ActErr
        - QueryId: Calculus
        - data: {code: ErrorCode, info: String}



### Calculations Events

### Errors Events

- **CommonFail**
    - Error
        - Content: Json
        - Cot: Info
        - QueryId: None
        - data: {code: ErrorCode, info: String}

- **SomeSpecificFail**
    - Error
        - Content: Json
        - Cot: Info
        - QueryId: SomeSpecificFail
        - data: {code: ErrorCode, info: String}
