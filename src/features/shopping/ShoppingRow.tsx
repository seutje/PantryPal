import { useState } from 'react';
import type { ShoppingItem, ShoppingOperation } from '../../lib/models';
import { Actions } from '../../components/ui/Actions';
import { Button } from '../../components/ui/Button';

export function ShoppingRow({
  item,
  disabled,
  onEdit,
  onApply,
}: {
  item: ShoppingItem;
  disabled: boolean;
  onEdit: () => void;
  onApply: (operation: ShoppingOperation) => void;
}) {
  const [reset, setReset] = useState(false);
  return (
    <article
      className="pp-shopping-row space-y-2"
      data-checked={item.checked}
      data-retained={item.excluded || item.orphaned}
      aria-label={item.label}
    >
      <div className="pp-shopping-line">
        <label className="pp-shopping-label">
          <input
            type="checkbox"
            checked={item.checked}
            disabled={disabled}
            onChange={(event) =>
              onApply({
                kind: 'check',
                itemId: item.id,
                checked: event.target.checked,
              })
            }
          />
          <span className="pp-item-name">{item.label}</span>
        </label>
        {item.adjustment && item.key && (
          <span className="pp-badge">Amount overridden</span>
        )}
        {!item.key && <span className="pp-badge">Manual item</span>}
        {item.checked && (
          <span className="pp-badge" aria-hidden="true">
            Purchased
          </span>
        )}
        {item.approximate && (
          <span className="pp-badge pp-badge-warning">Approximate</span>
        )}
        {item.rounded && (
          <span className="pp-badge pp-badge-warning">Rounded up</span>
        )}
        <span className="pp-shopping-amount">{item.display}</span>
        <Button
          variant="quiet"
          aria-label={`Edit amount for ${item.label}`}
          disabled={disabled}
          onClick={onEdit}
        >
          Edit amount
        </Button>
        <Actions accessibleLabel={`More actions for ${item.label}`}>
          <Button
            disabled={disabled}
            onClick={() =>
              onApply({
                kind: 'exclude',
                itemId: item.id,
                excluded: !item.excluded,
              })
            }
          >
            {item.excluded ? 'Include' : 'Exclude'} {item.label}
          </Button>
          {item.adjustment && item.key && (
            <Button disabled={disabled} onClick={() => setReset(true)}>
              Reset amount for {item.label}
            </Button>
          )}
        </Actions>
      </div>
      {item.alternatives.length > 0 && (
        <p className="pp-muted">{item.alternatives.join(' · ')}</p>
      )}
      {item.excluded && <span className="pp-badge">Excluded from copy</span>}
      {item.orphaned && (
        <p className="pp-muted">
          Source no longer present; retained for review and omitted from copy
        </p>
      )}
      {item.warnings.map((warning) => (
        <p key={warning} className="text-sm text-amber-800">
          {warning}
        </p>
      ))}
      {reset && (
        <div
          role="group"
          aria-label={`Confirm amount reset for ${item.label}`}
          className="pp-panel space-y-2"
        >
          <p>
            Reset this amount to the generated quantity? Your edited amount will
            be removed.
          </p>
          <div className="flex gap-2">
            <Button disabled={disabled} onClick={() => setReset(false)}>
              Keep edited amount
            </Button>
            <Button
              disabled={disabled}
              variant="danger"
              onClick={() => {
                onApply({ kind: 'reset', itemId: item.id, confirmed: true });
                setReset(false);
              }}
            >
              Confirm amount reset
            </Button>
          </div>
        </div>
      )}
      {item.sources.length > 0 && (
        <details>
          <summary className="pp-muted">
            Recipe quantities ({item.sources.length})
          </summary>
          <ul className="list-disc pl-5 pp-muted">
            {item.sources.map((source) => (
              <li key={`${source.entryId}:${source.line.id}`}>
                {source.mealTitle}: {source.line.rawText} · planned{' '}
                {source.plannedServings.numerator}/
                {source.plannedServings.denominator} servings, base{' '}
                {source.baseServings.numerator}/
                {source.baseServings.denominator}
                {source.line.optional ? ' · optional' : ''}
              </li>
            ))}
          </ul>
        </details>
      )}
    </article>
  );
}
