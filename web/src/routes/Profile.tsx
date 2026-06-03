import { createSignal, Show, type JSX } from "solid-js";
import { auth } from "../api";
import { ApiError } from "../api/client";
import { useI18n } from "../lib/i18n";
import { currentUser, setUser } from "../lib/session";
import { pushToast } from "../components/Toast";
import { Avatar } from "../components/Avatar";
import { PhotoCropper } from "../components/PhotoCropper";

function fileToDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(typeof r.result === "string" ? r.result : "");
    r.onerror = () => reject(r.error ?? new Error("read failed"));
    r.readAsDataURL(file);
  });
}

const MAX_BYTES = 5 * 1024 * 1024;

export default function ProfileView(): JSX.Element {
  const { t } = useI18n();
  const u = () => currentUser();

  const [displayName, setDisplayName] = createSignal(u()?.display_name ?? "");
  const [partnerName, setPartnerName] = createSignal(u()?.partner_name ?? "");
  const [busy, setBusy] = createSignal(false);
  const [uploading, setUploading] = createSignal(false);
  /** Source data URL of the file the user picked, awaiting crop confirmation. */
  const [cropSrc, setCropSrc] = createSignal<string | null>(null);

  async function saveProfile(e: Event): Promise<void> {
    e.preventDefault();
    setBusy(true);
    try {
      const res = await auth.updateProfile({
        display_name: displayName().trim() || undefined,
        partner_name: partnerName().trim() || null,
      });
      setUser(res.user);
      pushToast(t("profile.saved"), "ok");
    } catch (err) {
      pushToast(err instanceof ApiError ? `${err.code}: ${err.message}` : t("profile.saveFailed"), "error");
    } finally {
      setBusy(false);
    }
  }

  async function onFileSelected(e: Event): Promise<void> {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    // Always reset the input so re-picking the same file fires onChange again.
    const reset = (): void => {
      input.value = "";
    };
    if (!file) return;
    if (file.size > MAX_BYTES) {
      pushToast(`Max 5 MB`, "error");
      reset();
      return;
    }
    try {
      const dataUrl = await fileToDataUrl(file);
      setCropSrc(dataUrl);
    } catch {
      pushToast(t("profile.saveFailed"), "error");
    } finally {
      reset();
    }
  }

  async function onCropConfirm(croppedDataUrl: string): Promise<void> {
    setCropSrc(null);
    setUploading(true);
    try {
      await auth.uploadPhoto(croppedDataUrl);
      // Refetch /me to pull new photo_url into session cache.
      const me = await auth.me();
      setUser(me.user);
      pushToast(t("profile.saved"), "ok");
    } catch (err) {
      pushToast(err instanceof ApiError ? `${err.code}: ${err.message}` : t("profile.saveFailed"), "error");
    } finally {
      setUploading(false);
    }
  }

  return (
    <div class="mx-auto w-full max-w-2xl space-y-6 px-4 py-5">
      <h2 class="h-title">{t("profile.title")}</h2>

      <section class="card flex flex-col gap-4 p-5 sm:flex-row sm:items-center">
        <Avatar user={u()} size="lg" />
        <div class="flex-1">
          <p class="meta">{t("profile.photoHint")}</p>
          <label class="btn btn-ghost mt-3 cursor-pointer">
            <span class="material-symbols-rounded" style={{ "font-size": "18px" }}>
              {u()?.photo_url ? "edit" : "add_a_photo"}
            </span>
            {uploading() ? t("profile.saving") : u()?.photo_url ? t("profile.changePhoto") : t("profile.uploadPhoto")}
            <input
              type="file"
              accept="image/jpeg,image/png,image/webp,image/gif"
              class="hidden"
              onChange={(e) => void onFileSelected(e)}
            />
          </label>
        </div>
      </section>

      <form class="card flex flex-col gap-4 p-5" onSubmit={saveProfile} novalidate>
        <label>
          <span class="label">{t("profile.yourName")}</span>
          <input
            class="field"
            value={displayName()}
            onInput={(e) => setDisplayName(e.currentTarget.value)}
            autocomplete="name"
          />
        </label>

        <label>
          <span class="label">{t("profile.partnerName")}</span>
          <input
            class="field"
            value={partnerName()}
            onInput={(e) => setPartnerName(e.currentTarget.value)}
            placeholder={t("profile.partnerNamePlaceholder")}
            autocomplete="off"
          />
        </label>

        <button class="btn btn-primary self-start" type="submit" disabled={busy()}>
          {busy() ? t("profile.saving") : t("profile.save")}
        </button>
      </form>

      <Show when={cropSrc()}>
        {(src) => (
          <PhotoCropper
            src={src()}
            onConfirm={(dataUrl) => void onCropConfirm(dataUrl)}
            onCancel={() => setCropSrc(null)}
          />
        )}
      </Show>
    </div>
  );
}

