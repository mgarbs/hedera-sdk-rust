import HederaProtobufs
import NumberKit

extension Rational: ProtobufCodable where T == UInt64 {
    typealias Protobuf = Proto_Fraction

    init(fromProtobuf protobuf: Protobuf) {
        self.init(UInt64(protobuf.numerator), UInt64(protobuf.denominator))
    }

    func toProtobuf() -> Protobuf {
        .with { proto in
            proto.numerator = Int64(self.numerator)
            proto.denominator = Int64(self.denominator)
        }
    }
}
