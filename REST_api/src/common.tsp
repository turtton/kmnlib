import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi3";

using TypeSpec.Http;
using TypeSpec.Rest;

namespace Common {
    model Success {
        @statusCode
        statusCode: 200;
    }
    model InternalError {
        @statusCode
        statusCode: 400;
    }
    model NotFound {
        @statusCode
        statusCode: 404;
    }
    model Conflict {
        @statusCode
        statusCode: 409;
    }
}
