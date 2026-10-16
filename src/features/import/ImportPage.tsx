import { useDraftGuard } from '../../lib/drafts';
import { useState } from 'react';
import { useMutation } from '@tanstack/react-query';
import {
  inspectRecipeUrl,
  openRecipeBrowser,
  previewRecipeImport,
  confirmRecipeImport,
  resetRecipeBrowser,
  captureRecipeBrowser,
} from '../../lib/ipc';
import type {
  Catalog,
  FieldDefinition,
  Meal,
  MealDraft,
  ImportInput,
  ImportedRecipeDraft,
} from '../../lib/models';
import { MealEditor } from '../meals/MealEditor';
import { Button } from '../../components/ui/Button';
import { FormField } from '../../components/ui/FormField';

export function BrowserReset() {
  const [confirm, setConfirm] = useState(false);
  const reset = useMutation({
    mutationFn: () => resetRecipeBrowser(true),
    onSuccess: () => setConfirm(false),
  });
  return (
    <section className="space-y-2" aria-label="Recipe website sessions">
      <p>
        Recipe website cookies and login sessions stay in a separate local
        browser profile. Websites can expire logins or require MFA. Clearing
        sessions signs you out of these websites.
      </p>
      <Button onClick={() => setConfirm(true)}>Clear website sessions</Button>
      {confirm && (
        <div role="group" aria-label="Confirm session reset">
          <p>
            Clear all recipe website cookies, local storage and cache? This
            signs you out of recipe websites. Household data stays saved.
          </p>
          <Button
            variant="danger"
            disabled={reset.isPending}
            onClick={() => reset.mutate()}
          >
            Confirm clear sessions
          </Button>
          <Button onClick={() => setConfirm(false)}>Keep sessions</Button>
        </div>
      )}
      {reset.isSuccess && <p role="status">Website sessions cleared.</p>}
      {reset.error && <p role="alert">{reset.error.message}</p>}
    </section>
  );
}
export function ImportPage({
  catalog,
  fields,
  onSaved,
  onCancel,
  initialUrl = '',
}: {
  catalog: Catalog;
  fields: FieldDefinition[];
  onSaved: (meal: Meal) => void;
  onCancel: () => void;
  initialUrl?: string;
}) {
  const [url, setUrl] = useState(initialUrl);
  const [format, setFormat] = useState<ImportInput['format']>('manual');
  const [payload, setPayload] = useState('');
  const [title, setTitle] = useState('');
  const [servings, setServings] = useState('');
  const [ingredients, setIngredients] = useState('');
  const [instructions, setInstructions] = useState('');
  const [permitted, setPermitted] = useState(false);
  const [preview, setPreview] = useState<{
    source: ImportInput;
    draft: ImportedRecipeDraft;
  } | null>(null);
  const [reviewed, setReviewed] = useState(false);
  const [duplicate, setDuplicate] = useState(false);
  const [discard, setDiscard] = useState(false);
  const [actionId, setActionId] = useState(() => crypto.randomUUID());
  const [pendingSave, setPendingSave] = useState<{
    actionId: string;
    source: ImportInput;
    reviewed: MealDraft;
    confirmed: boolean;
    allowDuplicate: boolean;
  } | null>(null);
  const inspect = useMutation({
    mutationFn: (url: string) => inspectRecipeUrl(url),
  });
  const browser = useMutation({
    mutationFn: (url: string) => openRecipeBrowser(url),
  });
  const prepare = useMutation({
    mutationFn: (input: ImportInput) => previewRecipeImport(input),
    onSuccess: (draft, source) => {
      setPreview({ source, draft });
      setReviewed(false);
      setDuplicate(false);
      setActionId(crypto.randomUUID());
      setPendingSave(null);
    },
  });
  const capture = useMutation({
    mutationFn: ({ url, permitted }: { url: string; permitted: boolean }) =>
      captureRecipeBrowser(url, permitted),
    onSuccess: (result, request) => {
      if (request.url !== url) return;
      setPreview(result);
      setReviewed(false);
      setDuplicate(false);
      setActionId(crypto.randomUUID());
      setPendingSave(null);
    },
  });
  const save = async (draft: MealDraft) => {
    if (!preview || !reviewed)
      throw new Error(
        'Confirm that you reviewed quantities, units, servings and ingredient matches.',
      );
    const request = {
      actionId,
      source: preview.source,
      reviewed: draft,
      confirmed: true,
      allowDuplicate: duplicate,
    };
    // A failed response can follow a committed transaction. Retry identical
    // requests with their original ID; corrected drafts get a fresh action ID.
    if (
      pendingSave &&
      JSON.stringify({ ...request, actionId: pendingSave.actionId }) ===
        JSON.stringify(pendingSave)
    )
      return confirmRecipeImport(pendingSave);
    const next = pendingSave
      ? { ...request, actionId: crypto.randomUUID() }
      : request;
    setPendingSave(next);
    setActionId(next.actionId);
    return confirmRecipeImport(next);
  };
  useDraftGuard(
    !!preview ||
      !!title ||
      !!servings ||
      !!ingredients ||
      !!instructions ||
      !!payload ||
      url !== initialUrl,
  );
  return (
    <section className="space-y-4" aria-label="Import recipe">
      <h2 className="text-xl font-semibold">
        {preview ? 'Review import' : 'Import recipe'}
      </h2>
      <ol aria-label="Import progress" className="pp-stepper">
        <li aria-current={!preview ? 'step' : undefined}>1 / Source</li>
        <li aria-current={preview ? 'step' : undefined}>2 / Review</li>
        <li>3 / Save</li>
      </ol>
      {!preview ? (
        <>
          <FormField label="Recipe URL (HTTPS)">
            <input
              autoFocus
              value={url}
              maxLength={2048}
              onChange={(e) => {
                setUrl(e.target.value);
                inspect.reset();
              }}
            />
          </FormField>
          <div className="flex gap-2">
            <Button
              variant="primary"
              disabled={inspect.isPending}
              onClick={() => inspect.mutate(url)}
            >
              Check website
            </Button>
            <Button
              disabled={
                !inspect.isSuccess ||
                inspect.variables !== url ||
                browser.isPending
              }
              onClick={() => browser.mutate(url)}
            >
              Open recipe website
            </Button>
          </div>
          {inspect.variables === url && inspect.data && (
            <p role="status">{inspect.data}</p>
          )}
          {browser.isSuccess && (
            <p>
              Browse and sign in directly on the website. Once the recipe is
              visible, return here to read it for review. If website access is
              unavailable, use permitted manual content below.
            </p>
          )}
          <label className="flex gap-2">
            <input
              type="checkbox"
              checked={permitted}
              onChange={(e) => setPermitted(e.target.checked)}
            />
            I am permitted to import this supplied recipe content.
          </label>
          <Button
            disabled={
              !permitted ||
              !inspect.isSuccess ||
              inspect.variables !== url ||
              capture.isPending ||
              prepare.isPending
            }
            onClick={() => capture.mutate({ url, permitted })}
          >
            {capture.isPending ? 'Reading recipe…' : 'Read recipe from browser'}
          </Button>

          <details className="pp-panel" open>
            <summary className="font-semibold">
              Use manual entry or authorized JSON-LD
            </summary>
            <div className="space-y-4 mt-4">
              <FormField label="Import method">
                <select
                  value={format}
                  onChange={(e) =>
                    setFormat(e.target.value as ImportInput['format'])
                  }
                >
                  <option value="manual">Manual recipe text</option>
                  <option value="json_ld">Authorized JSON-LD</option>
                </select>
              </FormField>
              {format === 'json_ld' ? (
                <FormField label="Recipe JSON-LD">
                  <textarea
                    rows={8}
                    maxLength={262144}
                    value={payload}
                    onChange={(e) => setPayload(e.target.value)}
                  />
                </FormField>
              ) : (
                <>
                  <FormField label="Recipe title">
                    <input
                      value={title}
                      maxLength={200}
                      onChange={(e) => setTitle(e.target.value)}
                    />
                  </FormField>
                  <FormField label="Recipe servings">
                    <input
                      value={servings}
                      onChange={(e) => setServings(e.target.value)}
                    />
                  </FormField>
                  <FormField label="Recipe ingredients (one per line)">
                    <textarea
                      rows={6}
                      value={ingredients}
                      onChange={(e) => setIngredients(e.target.value)}
                    />
                  </FormField>
                  <FormField label="Recipe instructions">
                    <textarea
                      rows={6}
                      value={instructions}
                      onChange={(e) => setInstructions(e.target.value)}
                    />
                  </FormField>
                </>
              )}
              <p>
                Paste a single schema.org Recipe JSON object or graph, without
                script tags. Unknown amounts and original unit wording stay
                available for review. Login failures and unavailable metadata
                can use manual entry.
              </p>
              <Button
                disabled={!permitted || prepare.isPending || capture.isPending}
                onClick={() =>
                  prepare.mutate({
                    sourceUrl: url,
                    format,
                    permitted,
                    payload:
                      format === 'json_ld'
                        ? payload
                        : JSON.stringify({
                            title,
                            servings,
                            ingredients: ingredients
                              .split('\n')
                              .filter((s) => s.trim()),
                            instructions,
                          }),
                  })
                }
              >
                Preview recipe
              </Button>
            </div>
          </details>
          {[inspect.error, browser.error, prepare.error, capture.error]
            .filter(Boolean)
            .map((error, index) => (
              <p role="alert" key={index}>
                {error?.message} Your input is still here.
              </p>
            ))}
        </>
      ) : (
        <>
          <div className="pp-panel">
            <p className="break-all">Source: {preview.source.sourceUrl}</p>
            <p>
              Original yield: {preview.draft.originalYield || 'Unknown'} ·
              Attribution: {preview.draft.attribution || 'Not provided'}
            </p>
            <div className="rounded-lg bg-amber-50 p-3 my-3">
              <h3 className="font-semibold">
                Review quantities and ingredient matches
              </h3>
              <a href="#recipe-ingredients" className="underline">
                Go to ingredient review
              </a>
              {preview.draft.warnings.map((warning, index) => (
                <p key={index}>{warning}</p>
              ))}
            </div>
          </div>
          {preview.draft.duplicateTitles.length > 0 && (
            <p role="status">
              Already saved from this source:{' '}
              {preview.draft.duplicateTitles.join(', ')}. Cancel or explicitly
              save another meal.
            </p>
          )}
          <MealEditor
            initialDraft={preview.draft.meal}
            submitDisabled={
              !reviewed ||
              (preview.draft.duplicateTitles.length > 0 && !duplicate)
            }
            catalog={catalog}
            fields={fields}
            onSaved={onSaved}
            onCancel={onCancel}
            submitDraft={save}
            beforeActions={
              <div className="space-y-3">
                <label className="flex gap-2">
                  <input
                    type="checkbox"
                    checked={reviewed}
                    onChange={(e) => setReviewed(e.target.checked)}
                  />
                  I reviewed servings, quantities, regional units and ingredient
                  matches.
                </label>
                <label className="flex gap-2">
                  <input
                    type="checkbox"
                    checked={duplicate}
                    onChange={(e) => setDuplicate(e.target.checked)}
                  />
                  Save another meal if this source already exists.
                </label>
              </div>
            }
          />
        </>
      )}
      {!preview && (
        <>
          <Button onClick={() => setDiscard(true)}>Cancel import</Button>
          {discard && (
            <div role="group" aria-label="Discard import">
              <p>Discard this unsaved import?</p>
              <Button onClick={onCancel}>Discard import</Button>
              <Button onClick={() => setDiscard(false)}>Keep importing</Button>
            </div>
          )}
          <details className="pp-panel">
            <summary>Recipe website sessions</summary>
            <p className="pp-muted mt-3">
              You can also manage website sessions in Settings.
            </p>
            <BrowserReset />
          </details>
        </>
      )}
    </section>
  );
}
