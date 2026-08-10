import { useEffect, useMemo, useRef, useState } from "react";
import {
  Button,
  Group,
  Image,
  Modal,
  Select,
  SimpleGrid,
  Stack,
  Text,
  Textarea,
  TextInput,
} from "@mantine/core";
import { api } from "../lib/api";
import { GOOD_QUALITY_SIZE, useAdaptiveImageSize } from "../lib/useAdaptiveImageSize";

interface Props {
  opened: boolean;
  onClose: () => void;
  onChanged: () => void; // список компонентов на главном экране мог измениться
}

interface Snapshot {
  props: [string, string, string, string];
  imageBase64: string | null;
}

const emptySnapshot: Snapshot = { props: ["", "", "", ""], imageBase64: null };

const PREVIEW_BOX = GOOD_QUALITY_SIZE;
const DESCRIPTION_HEIGHT = 85;

export function EditorModal({ opened, onClose, onChanged }: Props) {
  const [names, setNames] = useState<string[]>([]);
  const [allProperties, setAllProperties] = useState<string[]>([]);

  const [loadedName, setLoadedName] = useState("");
  const [isNew, setIsNew] = useState(false);
  // Название/свойства/удаление разрешены только для ингредиентов, которые
  // сам пользователь добавил через "Новый" (Addon::UserAdded) — у
  // остальных нет безопасного пути восстановления при ошибке.
  const [editable, setEditable] = useState(true);

  const [newDialogOpen, setNewDialogOpen] = useState(false);
  const [newDialogGeneration, setNewDialogGeneration] = useState(0);
  const newNameRef = useRef<HTMLInputElement>(null);

  const [propSelects, setPropSelects] = useState<[string, string, string, string]>(["", "", "", ""]);
  const [imageBase64, setImageBase64] = useState<string | null>(null);
  const [imageFileName, setImageFileName] = useState("");

  // "Описание" — неконтролируемое поле (текст живёт в DOM через ref, а не в
  // React state): иначе каждое нажатие клавиши перерисовывало всю форму
  // редактора целиком, включая тяжёлые списки "Ингредиент"/"Свойство" —
  // именно это и вызывало ощутимые тормоза при печати.
  const [description, setDescription] = useState("");
  const [descriptionTouched, setDescriptionTouched] = useState(false);
  const descriptionRef = useRef<HTMLTextAreaElement>(null);

  const originalRef = useRef<Snapshot>(emptySnapshot);
  const [dirty, setDirty] = useState(false);

  const [info, setInfo] = useState<{ title: string; text: string } | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [confirmBaseEdit, setConfirmBaseEdit] = useState(false);
  const [pendingAction, setPendingAction] = useState<
    { kind: "load"; name: string } | { kind: "close" } | { kind: "new" } | null
  >(null);

  const previewSrc = useMemo(
    () => (imageBase64 ? dataUrlFromBase64(imageBase64) : null),
    [imageBase64],
  );
  const previewSize = useAdaptiveImageSize(previewSrc);

  useEffect(() => {
    if (!opened) return;
    api.getProperties().then(setAllProperties).catch(() => {});
    api
      .getComponentNames()
      .then((ns) => {
        setNames(ns);
        if (ns.length > 0) loadComponent(ns[0]);
        else clearFields();
      })
      .catch(() => {});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [opened]);

  function currentSnapshot(): Snapshot {
    return { props: propSelects, imageBase64 };
  }

  function recomputeDirty(next: Partial<Snapshot> = {}) {
    const cur = { ...currentSnapshot(), ...next };
    const orig = originalRef.current;
    const changed =
      descriptionTouched ||
      cur.imageBase64 !== orig.imageBase64 ||
      cur.props.some((p, i) => p !== orig.props[i]);
    setDirty(changed);
  }

  function clearFields() {
    setImageBase64(null);
    setImageFileName("");
    setDescription("");
    setDescriptionTouched(false);
    setPropSelects(["", "", "", ""]);
    setLoadedName("");
    setIsNew(false);
    setEditable(true);
    originalRef.current = emptySnapshot;
    setDirty(false);
  }

  async function loadComponent(name: string) {
    const [props, media, userAdded] = await Promise.all([
      api.getComponentPropertiesWithTypes(name),
      api.getComponentMedia(name),
      api.isUserAddedComponent(name),
    ]);
    const propNames = props.slice(0, 4).map((p) => p.name);
    while (propNames.length < 4) propNames.push("");
    const propsTuple = propNames as [string, string, string, string];
    const b64 = media.image_data_url ? base64FromDataUrl(media.image_data_url) : null;

    setPropSelects(propsTuple);
    setImageBase64(b64);
    setImageFileName("");
    setDescription(media.description);
    setDescriptionTouched(false);

    setLoadedName(name);
    setIsNew(false);
    setEditable(userAdded);
    originalRef.current = { props: propsTuple, imageBase64: b64 };
    setDirty(false);
  }

  function startNew(name: string) {
    setImageBase64(null);
    setImageFileName("");
    setDescription("");
    setDescriptionTouched(false);
    setPropSelects(["", "", "", ""]);
    setLoadedName(name);
    setIsNew(true);
    setEditable(true);
    originalRef.current = emptySnapshot;
    setDirty(false);
  }

  function requestAction(action: { kind: "load"; name: string } | { kind: "close" } | { kind: "new" }) {
    if (!dirty) {
      runAction(action);
    } else {
      setPendingAction(action);
    }
  }

  function runAction(action: { kind: "load"; name: string } | { kind: "close" } | { kind: "new" }) {
    if (action.kind === "load") loadComponent(action.name);
    else if (action.kind === "new") {
      setNewDialogGeneration((g) => g + 1);
      setNewDialogOpen(true);
    } else onClose();
  }

  async function confirmNewName() {
    const name = (newNameRef.current?.value ?? "").trim();
    if (!name) {
      setInfo({ title: "Ошибка", text: "Введите название ингредиента." });
      return;
    }
    if (await api.componentExists(name)) {
      setInfo({ title: "Ошибка", text: `Ингредиент «${name}» уже существует.` });
      return;
    }
    setNewDialogOpen(false);
    startNew(name);
  }

  async function pickImage() {
    const picked = await api.pickImageFile();
    if (!picked) return;
    setImageBase64(picked.base64);
    setImageFileName(picked.file_name);
    recomputeDirty({ imageBase64: picked.base64 });
  }

  function clearImage() {
    setImageBase64(null);
    setImageFileName("");
    recomputeDirty({ imageBase64: null });
  }

  async function handleSave() {
    const seen = new Set<string>();
    for (let i = 0; i < 4; i++) {
      const p = propSelects[i];
      if (!p) {
        setInfo({ title: "Ошибка", text: `Выберите значение во всех 4 полях «Свойство» (поле ${i + 1}).` });
        return;
      }
      if (seen.has(p)) {
        setInfo({ title: "Ошибка", text: "Свойства компонента не должны повторяться." });
        return;
      }
      seen.add(p);
    }

    // Для не-пользовательских ингредиентов свойства/название всё равно
    // заблокированы в форме — тут реально может меняться только картинка
    // и описание, но это тоже правка "базовых" данных, поэтому отдельное
    // подтверждение.
    if (!isNew && !editable) {
      setConfirmBaseEdit(true);
      return;
    }
    await performSave();
  }

  async function performSave() {
    const descriptionValue = (descriptionRef.current?.value ?? "").trim();
    try {
      if (isNew) {
        await api.insertComponent(loadedName, propSelects);
        const ns = await api.getComponentNames();
        setNames(ns);
      } else if (editable) {
        await api.updateComponentProperties(loadedName, propSelects);
      }
      await api.setComponentMedia(loadedName, imageBase64, descriptionValue);

      setIsNew(false);
      setDescription(descriptionValue);
      setDescriptionTouched(false);
      originalRef.current = { props: propSelects, imageBase64 };
      setDirty(false);
      onChanged();
      setInfo({ title: "Готово", text: "Изменения сохранены." });
    } catch (e) {
      setInfo({ title: "Ошибка", text: String(e) });
    }
  }

  async function handleDelete() {
    try {
      await api.deleteComponent(loadedName);
      const ns = await api.getComponentNames();
      setNames(ns);
      if (ns.length > 0) await loadComponent(ns[0]);
      else clearFields();
      onChanged();
      setInfo({ title: "Готово", text: "Компонент удалён." });
    } catch (e) {
      setInfo({ title: "Ошибка", text: String(e) });
    } finally {
      setConfirmDelete(false);
    }
  }

  return (
    <Modal opened={opened} onClose={() => requestAction({ kind: "close" })} title="Редактировать базу" size="xl">
      <Stack gap="sm">
        <div>
          <Text size="sm" fw={700} mb={2}>
            Ингредиент
          </Text>
          <Group wrap="nowrap" gap="xs" align="flex-start">
            {isNew ? (
              <TextInput flex={1} value={loadedName} disabled />
            ) : (
              <Select
                flex={1}
                data={names}
                value={names.includes(loadedName) ? loadedName : null}
                onChange={(v) => v && requestAction({ kind: "load", name: v })}
                searchable
                clearable={false}
                comboboxProps={{ withinPortal: true }}
              />
            )}
            <Button variant="default" onClick={() => requestAction({ kind: "new" })}>
              Новый
            </Button>
          </Group>
        </div>

        <SimpleGrid cols={2}>
          {[0, 1, 2, 3].map((i) => (
            <Select
              key={i}
              label={`Свойство ${i + 1}`}
              placeholder="— не выбрано —"
              data={allProperties}
              value={propSelects[i] || null}
              disabled={!editable}
              onChange={(v) => {
                const next = [...propSelects] as [string, string, string, string];
                next[i] = v ?? "";
                setPropSelects(next);
                recomputeDirty({ props: next });
              }}
              searchable
              clearable
              comboboxProps={{ withinPortal: true }}
            />
          ))}
        </SimpleGrid>

        <div>
          <Text size="sm" fw={700} mb={4}>
            Изображение
          </Text>
          <Group align="center" wrap="nowrap">
            <div
              style={{
                width: PREVIEW_BOX,
                height: PREVIEW_BOX,
                minWidth: PREVIEW_BOX,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                border: "1px solid var(--mantine-color-gray-4)",
                borderRadius: 6,
              }}
            >
              {previewSrc ? (
                <Image src={previewSrc} w={previewSize} h={previewSize} fit="contain" />
              ) : (
                <Text c="dimmed">нет</Text>
              )}
            </div>
            <Stack flex={1} gap={4}>
              <Button variant="light" onClick={pickImage}>
                Выбрать файл с изображением
              </Button>
              <Button variant="light" color="red" disabled={!imageBase64} onClick={clearImage}>
                Удалить изображение из базы
              </Button>
              {imageFileName && (
                <Text size="xs" c="dimmed">
                  {imageFileName}
                </Text>
              )}
            </Stack>
          </Group>
        </div>

        <Textarea
          key={loadedName}
          label="Описание"
          styles={{ input: { minHeight: DESCRIPTION_HEIGHT } }}
          ref={descriptionRef}
          defaultValue={description}
          onChange={() => {
            setDescriptionTouched(true);
            setDirty(true);
          }}
        />

        <Group justify="space-between" mt="xs">
          <Group>
            <Button
              variant="default"
              color="red"
              disabled={!loadedName || isNew || !editable}
              onClick={() => setConfirmDelete(true)}
            >
              Удалить
            </Button>
          </Group>
          <Group>
            <Button variant="default" onClick={() => requestAction({ kind: "close" })}>
              Закрыть
            </Button>
            <Button disabled={!dirty} onClick={handleSave}>
              Сохранить
            </Button>
          </Group>
        </Group>
      </Stack>

      <Modal opened={newDialogOpen} onClose={() => setNewDialogOpen(false)} title="Новый ингредиент" size="sm">
        <TextInput
          key={newDialogGeneration}
          label="Название"
          ref={newNameRef}
          defaultValue=""
          data-autofocus
        />
        <Group justify="flex-end" mt="md">
          <Button variant="default" onClick={() => setNewDialogOpen(false)}>
            Отмена
          </Button>
          <Button onClick={confirmNewName}>Создать</Button>
        </Group>
      </Modal>

      <Modal opened={confirmBaseEdit} onClose={() => setConfirmBaseEdit(false)} title="Подтверждение" size="sm">
        <Text size="sm" mb="md">
          Вы уверены, что хотите внести изменения в описание базового ингредиента?
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setConfirmBaseEdit(false)}>
            Отмена
          </Button>
          <Button
            onClick={() => {
              setConfirmBaseEdit(false);
              performSave();
            }}
          >
            Продолжить
          </Button>
        </Group>
      </Modal>

      <Modal opened={confirmDelete} onClose={() => setConfirmDelete(false)} title="Удаление компонента" size="sm">
        <Text size="sm" mb="md">
          Удалить компонент «{loadedName}»? Действие необратимо.
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setConfirmDelete(false)}>
            Отмена
          </Button>
          <Button color="red" onClick={handleDelete}>
            Удалить
          </Button>
        </Group>
      </Modal>

      <Modal
        opened={pendingAction !== null}
        onClose={() => setPendingAction(null)}
        title="Несохранённые изменения"
        size="sm"
      >
        <Text size="sm" mb="md">
          Данные не сохранены. Продолжить и потерять изменения?
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setPendingAction(null)}>
            Отмена
          </Button>
          <Button
            color="red"
            onClick={() => {
              const action = pendingAction;
              setPendingAction(null);
              if (action) runAction(action);
            }}
          >
            Продолжить
          </Button>
        </Group>
      </Modal>

      <Modal opened={info !== null} onClose={() => setInfo(null)} title={info?.title} size="sm">
        <Text size="sm">{info?.text}</Text>
      </Modal>
    </Modal>
  );
}

function base64FromDataUrl(dataUrl: string): string {
  const idx = dataUrl.indexOf(",");
  return idx === -1 ? dataUrl : dataUrl.slice(idx + 1);
}

function dataUrlFromBase64(base64: string): string {
  // На этапе редактирования формат не известен заранее (файл только что
  // выбран пользователем, без похода на бэкенд) — img тег определяет тип
  // по содержимому достаточно надёжно для локального превью, префикс здесь
  // не критичен.
  return `data:image/png;base64,${base64}`;
}
