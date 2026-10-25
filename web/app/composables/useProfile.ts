type ChangePasswordRequest = {
  current_password: string;
  new_password: string;
  password_confirmation: string;
};

export function useProfile() {
  const userStore = useUserStore();
  const requestFetch = useRequestFetch();

  async function updateProfile(email: string) {
    const user = await requestFetch<User>("/api/auth/me", {
      method: "PUT",
      body: { email },
    });
    userStore.setUser(user);

    return user;
  }

  function changePassword(body: ChangePasswordRequest) {
    return requestFetch("/api/auth/password", { method: "PUT", body });
  }

  return { updateProfile, changePassword };
}
