import "@typespec/http";
import "@typespec/rest";
import "@typespec/openapi3";
import "./common.tsp";
import "./rent.tsp";
import "./main.tsp";

using TypeSpec.Http;
using TypeSpec.Rest;

@tag("user")
@route("/users")
namespace KMNLib.UserAPI;
@format("uuid")
scalar UserId extends string;

model CreateUser {
  name: string;
  rent_limit: int32;
}

model UpdateUser {
  name?: string;
  rent_limit?: int32;
}

model UserResponse {
  id: UserId;
  name: string;
  rent_limit: int32;
}

model UserCreatedResponse {
  id: UserId;
}

interface Users {
  @summary("Get All Users")
  @get
  get(): {
    ...Common.Success;
    @body user: UserResponse[];
  } | Common.InternalError;

  @summary("Create user")
  @post
  post(
    @body
    body: CreateUser,
  ): {
    ...Common.Success;

    @body
    body: UserCreatedResponse;
  } | Common.InternalError;
}

@route("/{id}")
namespace User {
  @summary("Get user from id")
  @get
  op get(
    @path
    id: UserId,
  ): {
    ...Common.Success;

    @body
    body: UserResponse;
  } | Common.InternalError | Common.NotFound;

  @summary("Update user data")
  @patch
  op patch(
    @path
    id: UserId,

    @body
    body: UpdateUser,
  ): Common.Success | Common.InternalError;

  @summary("Delete user")
  @delete
  op delete(
    @path
    id: UserId,
  ): {
    ...Common.Success;
    @body body: UserId;
  } | Common.InternalError;

  @route("/rents")
  interface Rents {
    @summary("Get user rent book informations")
    @get
    get(
      @path
      id: UserId,
    ): {
      ...Common.Success;

      @body
      body: RentAPI.RentResponse;
    } | Common.InternalError | Common.NotFound;
  }
}
