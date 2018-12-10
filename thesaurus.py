from vocabulary.vocabulary import Vocabulary
import json


def synonyms(word):
   try:
      raw_json = Vocabulary.synonym(word)
      parsed = json.loads(raw_json)
      return sorted([item["text"] for item in parsed])

   except ValueError:
      print("Data was supplied in an invalid format (parse error)")
   except IndexError:
      print("Data was supplied in an invalid format (index error)")
   except KeyError:
      print("Data was supplied in an invalid format (key error)")
   
   return []

