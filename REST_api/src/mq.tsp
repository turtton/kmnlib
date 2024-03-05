import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi3";
import "./common.tsp";
import "./main.tsp";

using TypeSpec.Http;
using TypeSpec.Rest;

@tag("queue")
@route("/queue")
namespace KMNLib.QueueStateAPI;

@format("uuid")
scalar InfoId extends string;

enum InfoTarget {
  delayed,
  failed,
}

model GetInfos {
  target: InfoTarget;
  size?: int64 = 0;
  offset?: int64 = 0;
}

model GetInfo {
  target: InfoTarget
}

enum LengthTarget {
  ...InfoTarget,
  queued,
}

model GetInfoLength {
  target: LengthTarget;
}

model InfoResponse {
  id: InfoId;
  data: string;
  stack_trace: string;
}

model InfoLengthResponse {
  length: uint64;
}

@route("/infos")
namespace Infos {
  @get
  op get(
    @body
    body: GetInfos,
  ): {
    ...Common.Success;
    @body data: InfoResponse[];
  } | Common.InternalError;

  @route("/{id}")
  interface Info {
    @get
    get(
      @path
      id: InfoId,
      @body
      body: GetInfo
    ): {
      ...Common.Success;

      @body
      body: InfoResponse;
    } | Common.InternalError | Common.NotFound;
  }

  @route("/len")
  interface Length {
    @get
    get(
      @body
      body: GetInfoLength,
    ): {
      ...Common.Success;

      @body
      body: InfoLengthResponse;
    } | Common.InternalError;
  }
}
