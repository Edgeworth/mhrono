New types:

- Span - generic Span that can do anything
- SpanExc
- SpanInc

Endpoint<T>:

- Closed(p)
- Open(p, left)
- Unbounded(left)

trait EndpointConversion<T>
option because may not be able to convert, e.g. usize 0 -> -1

- to_open(p, left) -> Option<Endpoint<T>>
- to_closed(p, left) -> Option<Endpoint<T>>

impl for all T that returns None

move span operations into trait SpanOps:

- cover
- contains
- contains_span (impl SpanOps) -> bool
- is_empty
- intersect (impl SpanOps) -> Span
- range_ref
- to_span -> Span
- to_inc -> Option<SpanInc>, + use EndpointConversion<T>
- to_exc -> Option<SpanExc>, + use EndpointConversion<T>
- to_bounds() -> (Bound<T>, Bound<T>)
- to_bounds_ref() -> (Bound<&T>, Bound<&T>)
- to_range -> Option<Range>, + use EndpointConversion<T>
- to_range_inclusive -> Option<RangeInclusive>, + use EndpointConversion<T>
- to_range_from -> Option<RangeFrom>, + use EndpointConversion<T>
- to_range_to -> Option<RangeTo>, + use EndpointConversion<T>
- to_range_to_inclusive -> Option<RangeToInclusive>, + use EndpointConversion<T>
- to_range_full -> Option<RangeFull>

Span:

- exc: inc-exc interval
- inc: inc-inc interval
- point: point interval
- empty: empty interval

SpanExc:

- new: inc-exc interval
- inc: inc-inc interval use EndpointConversion<T>
- point: point interval use EndpointConversion<T>
- empty: empty interval
- as_range() -> Range

SpanInc:

- new: inc-inc interval
- exc: inc-exc interval use EndpointConversion<T>
- point: point interval
- empty: empty interval use EndpointConversion<T>
- as_range_inclusive() -> RangeInclusive

Implement from RangeBounds for Span + reverse

Implement from Range for SpanExc + reverse
Implement TryFrom RangeInclusive for SpanExc + reverse use EndpointConversion<T>

Implement from RangeInclusive for SpanInc + reverse
Implement TryFrom Range for SpanInc + reverse use EndpointConversion<T>
